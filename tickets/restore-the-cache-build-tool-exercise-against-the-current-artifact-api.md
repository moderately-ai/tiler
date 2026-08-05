---
id: restore-the-cache-build-tool-exercise-against-the-current-artifact-api
title: Restore the cache build-tool exercise against the current artifact API
status: in-progress
priority: p2
dependencies: []
related: [measure-the-expansion-cache-hot-path-efficiency, exercise-the-expansion-cache-under-cargo-and-rust-analyzer]
scopes: [research/cache]
shared_scopes: []
paths: []
tags: [cache, spike, maintenance]
claimed_from: todo
assignee: agent-cache-tool
lease_expires_at: 1785935822
---
## What is broken

`spikes/cache/build-tool-exercise/envelope` does not compile at `81a19a78`. Reproduce in one command:

```sh
cd spikes/cache/build-tool-exercise && cargo build -p exercise-envelope
```

Two drifts, both in `envelope/src/lib.rs`:

- `NumericalContract::FlushSubnormalsToZeroF32` no longer exists. `tiler_compiler::session::NumericalContract` is now a composed record with associated constants; the replacement is `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32`.
- `BackendEntryRef` has no `payload` field. It carries `payloads: Vec<PayloadId>`, one per delivery position, and the builder refuses an empty or cardinality-mismatched list by name.

Both fixes are one line each; `spikes/cache/hot-path-efficiency/harness/src/envelope.rs` compiles against the current API and is a working reference for the second one.

## Why the fix is not the whole ticket

`spikes/cache/README.md` publishes the exercise's 2026-07-25 and 2026-08-01 results as tracked evidence for [the build-tool exercise note](../docs/research/cache/build-tool-exercise.md), and that note's headline — twelve expansions producing four compilations, against twelve under the negative control — is a positive claim that outlives its producer. A retained result whose harness no longer builds is a claim nobody can re-derive. The README also records that this exact thing happened once before, at `63f9259`, and that the fix was followed by a re-run whose counted columns were compared row by row against the previous result.

That is the standard to repeat: a compile fix alone leaves the evidence in the same state this ticket found it in.

## Closes when

`cargo build` succeeds across the exercise workspace; the recorded driver invocation has been re-run and its result retained under `spikes/cache/results/` with the host and toolchain header; every counted column is compared against the 2026-08-01 row and any difference is explained rather than absorbed — including `negative-control-x3`, which is what makes the other rows mean anything; and `spikes/cache/README.md` records the re-run the way it records the previous two.
