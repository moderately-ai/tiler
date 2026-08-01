---
id: dispatch-a-tiler-region-on-metal-hardware
title: Complete one inline region's dispatch on Metal hardware
status: in-progress
priority: p1
dependencies: [route-an-embedded-artifact-through-a-consumer-storage-seam]
related: [route-an-embedded-artifact-through-a-consumer-storage-seam, prototype-candle-metal-adapter]
scopes: [implementation/frontend, research/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, inline-dx, runtime, metal, public-boundary]
claimed_from: todo
assignee: worker-hw-dispatch
lease_expires_at: 1785574184
---
## Why this exists

`route-an-embedded-artifact-through-a-consumer-storage-seam` built the storage seam and the committed outcome, and stopped one step short of a completed dispatch. That step is blocked by a *placement* decision rather than by unwritten code, which is why it is its own ticket.

**Fact.** `RouteOutcome::Dispatched` exists and nothing in the repository reaches it. Three placements were each eliminated by a check that is one line to reproduce:

1. A `trybuild` fixture cannot link Metal. `trybuild` generates its manifest from the crate under test's `[dependencies]`; inspect `target/tests/trybuild/tiler/Cargo.toml` after a run and it lists `tiler`, `tiler-artifact`, `tiler-ir`, `tiler-macros`, `tiler-runtime` and no `metal`. `tiler` must never carry a backend, so the fixture can never acquire one.
2. An integration test under `crates/tiler/tests/` cannot read a `MTLBuffer`. `metal` 0.33's only storage accessor is `Buffer::contents() -> *mut c_void`, and dereferencing it needs `unsafe`, which `[workspace.lints.rust] unsafe_code = "forbid"` in the root `Cargo.toml` makes unrelaxable by any inner attribute.
3. No workspace package may depend on `tiler` — `crates/tiler/tests/dependency_direction.rs::no_package_depends_on_the_frontend` — so `prototypes/serial-sum-run`, which already has `metal` and the two `unsafe` sites ADR 0079 admits, cannot be the consumer.

**Inference.** What is needed is an out-of-tree consumer crate that links `tiler` and `metal` and is not a workspace member — the shape twelve directories under `spikes/` already use, each carrying its own `[workspace]` table.

**Fact.** Each link already has evidence separately. The facade reaches a real routed entry against a real compiled metallib (that ticket's recorded transcript: entry symbol, 3859 object bytes, four bindings). `route_with_adapter` reaches a completed dispatch on Metal hardware in `prototypes/candle-metal-adapter`, and a completed dispatch with a bit-exact oracle in `crates/tiler-runtime/tests/adapter_route`. No single artifact shows the two composed.

## The decision this needs first

**Tom's, because it is a placement and a lint boundary, not an implementation detail.** Either:

- a spike under `spikes/runtime/` (or a new subject directory) with its own `[workspace]` table, run by hand from its own directory per AGENTS.md, linking `tiler` by path and `metal` — no workspace lint inheritance, so `unsafe` for the buffer readback is the spike's own admission under ADR 0079's four conditions; or
- a relaxation of the workspace `unsafe_code = "forbid"` for one named crate, which AGENTS.md reserves to Tom explicitly and which this ticket does *not* assume.

The first costs nothing already decided and is the recommendation. The second buys an in-gate test and spends the property that makes `forbid` meaningful.

## User-visible outcome

One inline `tiler::tensor!` invocation in an ordinary crate produces a running Metal kernel: the embedded artifact is routed, committed, and dispatched against the consumer's own values, and the result the consumer receives is the kernel's.

## Closes when

- A consumer crate depending on `tiler` and `metal` reaches `RouteOutcome::Dispatched` on this host, with the routing commit taken inside `route_with_adapter` and no fallback after it.
- A correctness oracle compares the dispatched bytes against the semantic fallback the consumer computes itself, bit for bit, **before** any performance claim is made. The oracle must not be derived from anything Tiler produced.
- The run is recorded with `tiler::__private::PRODUCER_DECLARED_EQUALITY` and says in those words that ADR 0086 refused the host, as `prototypes/serial-sum-run` does.
- A post-commit failure is watched failing: a perturbation that halts the submission must surface as `BindError::DispatchFailed` and must not return the semantic fallback's value.
- The spike's invocation is recorded in its own README and linked from the documents that cite it; no `make` target reaches it.

## Outcome

`spikes/runtime/inline-dispatch/` is the consumer, and every Closes-when clause is met on this host. Its [README](../spikes/runtime/inline-dispatch/README.md) carries the exact invocation, the host and toolchain table, both verbatim transcripts, and the perturbation table.

### The placement decision, resolved by elimination rather than escalated

The ticket reserved this to Tom and named two options. **The elimination has one survivor, so there was no question to ask.** Option 2 — relaxing the workspace `unsafe_code = "forbid"` for one named crate — is itself an act AGENTS.md reserves to Tom, so it is not an option a worker may *choose*; proposing it is the most a ticket can do. Option 1 — an out-of-tree spike with its own `[workspace]` table — spends nothing already decided, is the ticket's own recommendation, and is an additive directory that reverts by deletion. With option 2 unchoosable, option 1 is the only candidate that survives, and asking would have spent Tom's time on a decision the constraints already made.

The coordinator resolved it under the overnight operating mode and it reaches Tom in the morning packet. Nothing about option 2 is foreclosed: an in-gate test remains available if Tom later relaxes `forbid`, and the spike is what would be superseded.

**Correction to the dispatch brief.** It stated that spikes are outside the scope map. They are not: `ticketsplease.toml:72` maps `research/runtime` to `["docs/research/runtime/**", "spikes/runtime/**"]`, so `tkt guard` reported `UNDER-DECLARED: research/runtime` against the declared pair. `research/runtime` was added to this ticket before the guard passed, which is what AGENTS.md asks for — a scope added before a mapped contract area is touched, rather than a `paths` entry standing in for one. `implementation/frontend` stays declared and shows as unaffected: the seam was already complete and `crates/tiler` needed no change, which is itself the result the ticket predicted.

### What the run establishes

**Fact.** One `tiler::tensor! { in a: f32[4], b: f32[4], c: f32[4]; deliver macos; out (a * b) + c }` in an out-of-tree crate depending on `tiler` (by path) and `metal` 0.33.0 reached a completed dispatch on an Apple M4 Max under macOS 27.0 (build 26A5388g), nightly-2026-07-19, Apple metal 32023.883. The routed entry is `tiler_kernel_ce0acbceb6c201da`, 3859 object bytes, four bindings, launch 4×1 — the same entry `route-an-embedded-artifact-through-a-consumer-storage-seam` recorded from the `trybuild` fixture, now executed rather than refused.

**The commit is `route_with_adapter`'s and no fallback follows it.** The spike reimplements no part of the driver. `route_with_adapter` calls `Preflight::commit()` on the line before it calls `RuntimeAdapter::dispatch`, and nothing else calls that method, so the recorded `dispatch` stage *is* the commit; the `committed route completed: 1/1 entry(ies) encoded, terminal status Completed` note exists only if that method returned `Ok`, which is the condition that makes the facade's outcome `RouteOutcome::Dispatched`. The binary refuses the run when either is absent.

**Fact — the oracle agrees bit for bit, and it can disagree.** `main::oracle` is plain Rust `f32` written as the region reads, deliberately not `mul_add`; it is derived from nothing Tiler produced. The comparison is byte-for-byte over the native-endian run and is the first claim the binary makes. Two perturbations were applied, run, and reverted: negating the addend printed `ORACLE DISAGREES: the kernel wrote [6.5, -5.0, -2.0, 1.0] and this consumer's own arithmetic gives [5.5, -7.0, -6.0, 7.0]` (exit 1), and removing the readback printed `ORACLE DISAGREES: the kernel wrote [0.0, 0.0, 0.0, 0.0]` (exit 1) — the second is what proves the kernel's answer reaches the consumer through the readback and not from anywhere else.

**Finding — the facade's fallback is not a second value oracle.** `bind_and_build` checks the operands and calls the adapter's `build` for the declared result; nothing in the facade evaluates the expression on the host. The binary runs the same region without `deliver` and reports what that establishes (rank, stored scalar, extents) and what it does not, printing its storage as `[0.0, 0.0, 0.0, 0.0]` so no reader mistakes it for a computed comparison. The Closes-when clause's "semantic fallback the consumer computes itself" is therefore the consumer's own arithmetic, which is what the oracle is.

**The diagnostic is printed in the constant's own words.** `tiler::__private::producer_declared_equality` renders `DIAGNOSTIC — producer-declared equality against tiler.metal.macos-apple9.msl4-0.f32.v1, NOT host-earned eligibility`, followed by a sentence naming ADR 0086's refusal of this host.

**The post-commit failure was watched failing.** `--halt-after-commit` encodes the route exactly as the sound path does and withholds the submission, leaving the command buffer live and non-terminal — a state reachable only after `Preflight::commit`, because `RuntimeAdapter::dispatch` is where it lives. It surfaces as `BindError::DispatchFailed`: `the committed route did not complete, and no fallback follows: metal.dispatch: the command buffer is NotEnqueued, which is not a terminal state, so nothing was read back`. **No value is returned at all**, so neither the halted result storage nor the semantic fallback's value could be mistaken for an answer. The perturbation is behind a flag and the checked-in default is sound. The terminal `Error` state is deliberately not injected — provoking a GPU fault risks a device reset and would not reproduce — and that boundary is recorded in the README rather than left as apparent coverage.

**The one `unsafe` site** is `src/buffer.rs::read_into`, admitted under ADR 0079 with all four conditions in the source: no safe route (`metal` 0.33.0 publishes only `Buffer::contents() -> *mut c_void`, `metal-0.33.0/src/buffer.rs:24`, and the upload half needs no site because `Device::new_buffer_with_data` is safe), an `#[allow(unsafe_code, reason = …)]`, an assertion against the buffer's own `length()`, and a `SAFETY` comment. The crate sets `unsafe_code = "deny"` so every other site is a hard error.

### Gate

`make full` is untouched: the spike is not a workspace member, carries its own `[workspace]` table, and no `make` target reaches it. Verified by running the full gate on the tree with the spike present. The spike's own `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean; it deliberately carries no `rust-toolchain.toml`, so rustup resolves the repository pin by ancestry. Its `target/` is already covered by the root `.gitignore`'s unanchored `target/` rule (`git check-ignore -v` names `.gitignore:13`), so no rule was added; its `Cargo.lock` is tracked, as every other spike workspace's is.

### Filed out of scope

**A consumer cannot answer a live-device route requirement.** The region delivered here declares none — `observe-live-device` is absent from the stage list, which is what zero rows looks like — so the spike answers `LiveDeviceObservation::Unrecognized` for both arms, which is fail-closed. The `tiler.metal.route-requirement.minimum-gpu-family` row *is* answerable from `MTLDevice::supportsFamily`, but its payload vocabulary is `tiler_metal::applicability::MetalGpuFamily` and a consumer may not name an internal crate; spelling the family names again in the spike would mint a second authority over a governed vocabulary, so it declines. **Any consumer whose region declares that row will be refused until the vocabulary is reachable through the facade**, which is a public-boundary question for Tom rather than something to work around locally. Not absorbed here.

**`docs/` was not edited.** This ticket's declared scope is `implementation/frontend` and `project/tickets`; the spike is linked from `spikes/runtime/README.md` (its subject index) and from this ticket. Whether `docs/integration/frontends.md` should cite it is a documentation-scope change.

### Measurement boundary

One host, one OS build, one device, one Metal compiler, one region: one entry, four bindings, no shared allocations. The multi-entry and shared-allocation paths in `plan_dispatch` are compiled but unexercised here; `crates/tiler-runtime/tests/adapter_route` is where those have watched evidence. Nothing is timed and no performance claim is made. A completed dispatch is not eligibility: ADR 0086 keeps native `metallib` translation `Unknown` on every macOS row currently observable.
