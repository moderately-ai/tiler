---
schema: "tiler-doc/v1"
id: "tiler.spike.runtime.inline-dispatch"
kind: "experiment"
title: "One inline region dispatched on Metal hardware"
topics: ["runtime", "inline-dx", "metal", "dispatch", "artifacts", "numerics"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["executable-model", "bounded-measurement"]
supports: ["tiler.research.runtime.execution-contract"]
entrypoints: ["spikes/runtime/inline-dispatch/src/main.rs", "spikes/runtime/inline-dispatch/src/adapter.rs", "spikes/runtime/inline-dispatch/src/buffer.rs"]
last_verified: "2026-08-01"
verified_at_commit: "8366ecd"
ticket: "dispatch-a-tiler-region-on-metal-hardware"
---

# One inline region dispatched on Metal hardware

An ordinary crate writes

```rust
let d = tiler::tensor! {
    in a: f32[4], b: f32[4], c: f32[4];
    deliver macos;
    out (a * b) + c
};
```

and receives what a Metal kernel wrote. The artifact the expansion embedded is decoded, routed, committed, and dispatched on this host's device through the `tiler::value::DispatchAdapter` seam, and the bytes the consumer gets back equal its own `f32` arithmetic bit for bit.

Every link had evidence separately before this. `crates/tiler/tests/facade/pass/inline_region_dispatches.rs` drives the facade to a real routed entry against a real compiled `metallib` and stops there, because a `trybuild` fixture cannot link Metal. `prototypes/candle-metal-adapter` and `crates/tiler-runtime/tests/adapter_route` each reach a completed dispatch through `route_with_adapter`, one on hardware and one against a bit-exact host oracle. **No artifact showed the two composed**, and `RouteOutcome::Dispatched` was reachable from nothing in the repository. This is that artifact.

## Why it is out of tree

Two properties of the workspace, each one line to reproduce, and neither is a limitation to work around:

- `crates/tiler/tests/dependency_direction.rs::no_package_depends_on_the_frontend` forbids any workspace package from depending on `tiler`, so no member crate may be the consumer.
- the root manifest's `[workspace.lints.rust] unsafe_code = "forbid"` cannot be relaxed by an inner attribute at any scope, and `metal` 0.33.0's only storage accessor is `Buffer::contents() -> *mut c_void`, so no member crate may read what a kernel wrote.

Relaxing the workspace `forbid` for one named crate is the other placement the ticket named, and it is not this spike's to choose: AGENTS.md reserves that to Tom. A separate workspace costs nothing already decided, so this crate carries its own `[workspace]` table and sets `unsafe_code = "deny"` — a hard error everywhere except the one site that opts in by name.

## Running it

By hand, from this directory. **No `make` target reaches a spike**, and none should: `make full` builds the workspace, and this crate is not a member of it.

```sh
cd spikes/runtime/inline-dispatch
cargo run --release
cargo run --release -- --halt-after-commit
```

The first is the sound run. The second is the post-commit perturbation described below; it is a flag rather than the default, so the checked-in state is sound.

Both exit `0` on success and `1` on any disagreement, including an oracle mismatch. `cargo build` needs the pinned toolchain (resolved by directory ancestry — this spike deliberately carries no `rust-toolchain.toml`) and the Apple Metal toolchain, because the `deliver macos;` expansion compiles the region ahead of time.

## Host and toolchain

| | |
|---|---|
| host | Apple M4 Max, arm64 |
| OS | macOS 27.0, build 26A5388g |
| Rust | `nightly-2026-07-19`, from the repository pin |
| Metal compiler | Apple metal version 32023.883 (`metalfe-32023.883`), target `air64-apple-darwin27.0.0` |
| `metal` crate | 0.33.0, the version the root `[workspace.dependencies]` pins |
| repository commit | `29a9680` plus `reconcile-the-pre-commit-allocation-seam-with-adr-0051` |

## Transcript, 2026-08-01

Verbatim, `cargo run --release`. Re-recorded when
[`reconcile-the-pre-commit-allocation-seam-with-adr-0051`](../../../tickets/reconcile-the-pre-commit-allocation-seam-with-adr-0051.md)
split the seam's sizing stage from its allocating one, which is why
`allocate-dispatch` appears between `plan-dispatch` and `dispatch`.

**Measurement — the entry symbol drifted, and not from this change.** It was
`tiler_kernel_ce0acbceb6c201da` when this record was written at `8366ecd` and is
`tiler_kernel_ae031ce7240f7495` at the base above. The object length, binding
count, launch, and every value are unchanged. The one-line check that places the
drift elsewhere: `git status --porcelain -- crates/tiler-compiler crates/tiler-ir
crates/tiler-macros crates/tiler-metal crates/tiler-metal-aot crates/tiler-build
crates/tiler-artifact` reports nothing on this branch, so the kernel identity's
inputs were untouched by it and the value moved with work landed between the two
commits. Re-running this spike is what detects such drift, which is the trade
[AGENTS.md](../../../AGENTS.md) records for keeping a cited transcript.

```
device: Apple M4 Max
mode: sound
oracle: the dispatched bytes equal this consumer's own f32 arithmetic bit for bit: [6.5, -5.0, -2.0, 1.0]
commit: committed route completed: 1/1 entry(ies) encoded, terminal status Completed, profile tiler.metal.macos-apple9.msl4-0.f32.v1
result: f32[4], 16 byte(s)
stage: bind
stage: validate-payload
stage: prepare-entries
stage: observe-prepared-entry
stage: plan-dispatch
stage: allocate-dispatch
stage: dispatch
handover: a = [1.5, -2.0, 0.25, 8.0]
handover: b = [4.0, 3.0, -16.0, 0.5]
handover: c = [0.5, 1.0, 2.0, -3.0]
handover: out = 16 byte(s) to write
entry 0: symbol "tiler_kernel_ae031ce7240f7495", 3859 object byte(s), 4 binding(s), launch 4×1
plan: 1 entry(ies), 0 shared allocation(s)
committed route completed: 1/1 entry(ies) encoded, terminal status Completed, profile tiler.metal.macos-apple9.msl4-0.f32.v1
DIAGNOSTIC — producer-declared equality against tiler.metal.macos-apple9.msl4-0.f32.v1, NOT host-earned eligibility
ADR 0086 refuses the host: native `metallib` translation during pipeline creation is a capability fact whose authority is Unknown on every macOS row currently observable, so no host — this one included — earns the right to offer `tiler.metal.macos-apple9.msl4-0.f32.v1`. The route above was settled on producer-declared equality, NOT host-earned eligibility.
fallback-only region: same declared interface (f32[4], 16 byte(s)), and its storage is [0.0, 0.0, 0.0, 0.0] — the facade constructs the declared result and evaluates nothing, so this is not a second value oracle
```

The entry symbol and object length agree with the routed entry `route-an-embedded-artifact-through-a-consumer-storage-seam` recorded from the `trybuild` fixture — `tiler_kernel_ce0acbceb6c201da`, 3859 object bytes, four bindings — which is the same region compiled by the same expansion, now executed rather than refused.

## What the commit evidence actually is

`RouteOutcome::Dispatched` is not returned to a consumer: `bind_route_and_build` yields the region's *value*, because that is what `let d = tiler::tensor! { … }` asked for. So the commit is established structurally instead.

`route_with_adapter` calls `Preflight::commit()` on the line before it calls `RuntimeAdapter::dispatch`, and nothing else calls that method. The `dispatch` stage appearing in the recorded stage list is therefore the routing commit, taken inside the driver that owns it — this spike reimplements no part of it and takes no fallback after it. The `committed route completed: …` note exists only if `dispatch` returned `Ok`, which is exactly the condition that makes the facade's outcome `RouteOutcome::Dispatched`; `main.rs` refuses the run when either is absent.

## The oracle

`main::oracle` is plain Rust `f32`, written the way this consumer would have written the region without Tiler. It is derived from nothing Tiler produced — not a reference kernel, not a sidecar, not the facade's fallback — because an oracle derived from the thing under test agrees with it by construction. The comparison is byte-for-byte over the native-endian `f32` run and runs **before** any other claim the binary makes.

It is deliberately not `mul_add`: the region declares `(a * b) + c`, a multiply and an add with a rounding between them, and a fused multiply-add rounds once and can differ in the last bit. The operands are chosen so every product and sum is exactly representable, which is what makes a bit-for-bit comparison a statement about the dispatch rather than about rounding.

**The facade's fallback is not a second oracle, and the run says so.** `tiler::__private::bind_and_build` checks the region's operands and calls the adapter's `build` for the declared result; nothing in the facade evaluates the expression on the host. The binary runs the same region without `deliver` and reports what that establishes — the same rank, stored scalar, and extents — and reports its storage as `[0.0, 0.0, 0.0, 0.0]` so no reader mistakes it for a computed comparison.

## The post-commit failure, watched failing

`--halt-after-commit` selects `adapter::Perturbation::HaltAfterCommit`. It perturbs the *adapter* and nothing else: the same device, the same region, the same operands, and the same encode. What it withholds is the submission — the command buffer is neither committed nor waited — which leaves it live and non-terminal. That state is reached only after `Preflight::commit`, because `RuntimeAdapter::dispatch` is where it lives.

The terminal `Error` state is deliberately **not** injected, and the boundary is stated rather than left as apparent coverage: forcing a command buffer into `Error` means provoking a GPU fault, which risks a device reset and would not reproduce. The `Error` arm is classified by `adapter::submission_outcome`, whose match over `MTLCommandBufferStatus` is exhaustive and wildcard-free.

Verbatim, `cargo run --release -- --halt-after-commit`, first four lines:

```
device: Apple M4 Max
mode: perturbed — the submission is halted after the routing commit
post-commit failure, as required: adapter.dispatch: the committed route did not complete, and no fallback follows: metal.dispatch: the command buffer is NotEnqueued, which is not a terminal state, so nothing was read back
no value was returned: the halted dispatch's result storage never reached the caller, so nothing could be mistaken for the semantic fallback's answer
```

The rest of the output is identical to the sound run except that the `committed route completed` note is absent, which the binary checks. Three properties hold together and the run refuses if any does not: the route still reached the `dispatch` stage, so this is a *post*-commit case rather than a refusal; the region surfaced `BindError::DispatchFailed`; and no value was returned at all, so neither the halted result storage nor the semantic fallback's value could be mistaken for an answer.

## Every check was watched failing

Each perturbation below was applied to the working tree, run, and reverted.

| check | perturbation | observed |
|---|---|---|
| the oracle | `left * right + addend` → `left * right - addend` | `ORACLE DISAGREES: the kernel wrote [6.5, -5.0, -2.0, 1.0] and this consumer's own arithmetic gives [5.5, -7.0, -6.0, 7.0]`, exit `1` |
| the readback delivers the value | `buffer::read_into` removed from `dispatch` | `ORACLE DISAGREES: the kernel wrote [0.0, 0.0, 0.0, 0.0] and this consumer's own arithmetic gives [6.5, -5.0, -2.0, 1.0]`, exit `1` |
| the post-commit refusal | `--halt-after-commit` | `BindError::DispatchFailed`, no value returned, exit `0` for the perturbed mode |

The second is what proves the kernel's answer reaches the consumer through the readback and not from anywhere else: without it the region returns the fallback's zero-filled declared result. There is deliberately **no** separate "the result is not all zeros" check, because the oracle already refuses that state for this region and a check that cannot reach a verdict the first did not reach is a check nothing could watch fail.

## The one `unsafe` site

`src/buffer.rs::read_into`, and nothing else. ADR 0079's four conditions are each visible in that file:

1. **No safe route through the foreign API.** `metal` 0.33.0 publishes exactly one storage accessor, `Buffer::contents(&self) -> *mut std::ffi::c_void` (`metal-0.33.0/src/buffer.rs:24`) — no slice accessor, no typed view, no copy-out. The *upload* half needs no site at all, because `Device::new_buffer_with_data` (`metal-0.33.0/src/device.rs:1956`) is a safe function.
2. **An `#[allow(unsafe_code, reason = …)]`**, naming why the site exists and what bounds it.
3. **An assertion against the foreign object's own report** — `buffer.length()`, not a length this crate computed twice, which is the disagreement a read past the mapping is made of.
4. **A `SAFETY` comment** naming the pointer's validity extent, the plain-old-data element type, the non-overlap, and the happens-before the observed `Completed` status supplies.

Every other Metal call this spike makes — device creation, library loading, function lookup, pipeline construction, allocation, encoding, dispatch, submission, waiting, and the terminal-status read — is a safe call and lives in `src/adapter.rs`.

## Unsupported cases and measurement boundary

- **Live-device route requirements are refused, not answered, and that is now a deliberate interim rather than an open gap.** The region delivered here declares none — `observe-live-device` is absent from the stage list, which is what zero rows looks like — so nothing in that method ran on this transcript. Both arms answer `LiveDeviceObservation::Unrecognized`, which is fail-closed: the loader refuses the route and the region takes its declared result. The `tiler.metal.route-requirement.minimum-gpu-family` row *is* answerable from `MTLDevice::supportsFamily`, but its payload vocabulary is `tiler_metal::applicability::MetalGpuFamily` and a consumer may not name an internal crate; spelling the family names again here would mint a second authority over a governed vocabulary. [Backend-scoped route-requirement answers](../../../docs/research/runtime/backend-scoped-route-requirement-answers.md) is the design record that derives the channel, and it records fail-closed as the explicit interim while the design is unimplemented — so `Unrecognized` here is the correct answer rather than a placeholder, and it stays correct until a backend publishes an answer surface a consumer may reach. **That surface is a public-boundary question for Tom rather than something to work around locally**; the record's own finding is that the neutral answer channel already exists and works, and what a consumer is missing is the payload decoder, not a channel.
- **One entry, four bindings, no shared allocations.** The multi-entry and shared-allocation paths in `plan_dispatch` and `allocate_dispatch` are written and compiled but not exercised by this region; `crates/tiler-runtime/tests/adapter_route` is where those paths have watched evidence, against a host interpreter.
- **The post-commit allocation failure is unwatched here.** `DispatchFailure::UndersizedStorage` is reachable only from an allocator that returns less than a length it accepted, and Metal's does not on this host — provoking one is not something this spike can do without lying about what it allocated. `crates/tiler-runtime/tests/adapter_route::a_shared_allocation_shorter_than_the_plan_sized_it_fails_after_the_commit` is where that classification has watched evidence, against a host interpreter whose allocator this repository controls. What this run does establish about the split is the *stage order*: `allocate-dispatch` appears after `plan-dispatch` and before `dispatch` in the transcript above, which is the ordering ADR 0051 asks for.
- **No performance claim.** Nothing here is timed, warmed up, or repeated. It is a correctness artifact.
- **One host.** Every statement above is about the machine in the table, at that OS build, with that Metal compiler and that device. `metallib` translation during pipeline creation remains `Unknown` under ADR 0086 on every macOS row currently observable, so a completed dispatch is not eligibility and this spike claims none.
