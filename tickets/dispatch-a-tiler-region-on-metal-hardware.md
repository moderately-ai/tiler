---
id: dispatch-a-tiler-region-on-metal-hardware
title: Complete one inline region's dispatch on Metal hardware
status: in-progress
priority: p1
dependencies: [route-an-embedded-artifact-through-a-consumer-storage-seam]
related: [route-an-embedded-artifact-through-a-consumer-storage-seam, prototype-candle-metal-adapter]
scopes: [implementation/frontend]
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
