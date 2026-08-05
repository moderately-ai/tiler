---
id: restore-the-cache-build-tool-exercise-against-the-current-artifact-api
title: Restore the cache build-tool exercise against the current artifact API
status: review
priority: p2
dependencies: []
related: [measure-the-expansion-cache-hot-path-efficiency, exercise-the-expansion-cache-under-cargo-and-rust-analyzer]
scopes: [research/cache]
shared_scopes: [project/tickets]
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

## Outcome

Done at base `9ec8028c`. The failure was re-reproduced at that base rather than transcribed from `81a19a78`, and it is still exactly the two drifts recorded above — no third had accumulated. Both replacements were read at their definitions (`crates/tiler-compiler/src/session.rs:1356`, `crates/tiler-artifact/src/program/model.rs:644`) rather than copied from the reference harness.

The recorded driver invocation was re-run in full, all eight scenarios passing, and retained as `spikes/cache/results/build-tool-exercise-macos-27.0-2026-08-05.tsv`:

```sh
python3 spikes/cache/build_tool_exercise.py --concurrency 3 \
  --analyzer "$(rustup which --toolchain nightly rust-analyzer)" \
  --record macos-27.0-2026-08-05
```

Seventy-two counted cells — events, builds, published, hit, uncached, processes, cwds, drivers, and the note, over all eight scenarios — are identical to the 2026-08-01 row, `negative-control-x3`'s twelve compilations included, so no counted quantity needed explaining. Only `overlaps` and `seconds` moved, which the README already records as quantities the driver reports rather than asserts on. The comparison was proved able to fail: forcing `negative-control-x3.builds` from 12 to 4 in a scratch copy made it report `DIFFERS` and exit non-zero.

The research note is deliberately untouched. Its Section 4 is a recorded environment for the 2026-07-25 run, and the note's own Section 8 argues that a recorded environment must keep saying what was true when it was recorded; the 2026-08-01 restoration set the same precedent, touching only `spikes/cache/README.md` beside the fix and the new result.

`project/tickets` was added to `shared_scopes` because the ticket's own body is edited here and every claimed ticket declares it.
