---
id: restore-the-scalar-cpu-vertical-spike-against-the-current-crates
title: Restore the scalar CPU vertical spike against the current crates
status: todo
priority: p2
dependencies: []
related: [generalize-payload-provenance-beyond-the-apple-shape, prototype-a-bounded-scalar-cpu-backend-vertical]
scopes: [research/target-profiles]
shared_scopes: [project/tickets]
paths: []
tags: [spikes, cpu, maintenance]
---
## User-visible outcome

The retained scalar CPU vertical compiles and runs again against the current `crates/`, and the result fixture it cites is a measurement of the code beside it rather than of a superseded one.

## Why this exists

**Fact — the spike does not compile, and it did not compile before the provenance work touched it.** At `cbec2d4`, `CARGO_TARGET_DIR=./target cargo check` from `spikes/target-profiles/scalar-cpu-vertical` fails with 13 errors. Ten of them are unrelated to payload provenance and are the subject of this ticket:

- `BackendEntryRef` has no field named `payload` — it is now `payloads`, from the delivery-position step (`src/vertical.rs:379`).
- `DecodedProgram::decode` takes two arguments — it gained a `delivery: usize` parameter (`crates/tiler-runtime/src/load.rs:229`) — at nine call sites in `src/vertical.rs`.

The remaining three were the provenance fields, and [`generalize-payload-provenance-beyond-the-apple-shape`](generalize-payload-provenance-beyond-the-apple-shape.md) fixed those in the same commit that made them necessary: the spike now states `PayloadPlatform::Unversioned`. It stopped there deliberately — repairing the delivery-position drift is a different change, needs a decision about which delivery position the spike's single-position artifact resolves, and would rebaseline cited evidence that the provenance ticket had no mandate to move.

**Fact — the result fixture is therefore stale in a way its own README now records.** `results/2026-07-31-macos-arm64.json` states `payload_bytes: 265`, `envelope_bytes: 20953`, and `artifact_identity_bytes: 9753`. The payload's canonical subject shrank by the three SDK text runs and grew by one appended platform tag, so at least the first of those is wrong, and the other two fold it. No number was hand-edited: a computed measurement is not a measurement.

**Inference — this is what a retained spike costs, and the cost is the point.** `AGENTS.md` records that only re-running a spike detects drift from the source beside it. Two API steps landed without it, and nothing reported that until a third change tried to compile it.

## Closes when

`cargo check` and `cargo run` both succeed from the spike's own directory under the invocation its README records; the run writes `results/` and every byte count in the fixture is from that run; the README's `last_verified` and `verified_at_commit` name the commit that ran it; and finding 7's closure note stops disclaiming the fixture because the fixture is current.

## Graph maintenance

File any further API drift found while repairing this as its own ticket rather than absorbing it — the value of this exercise is the enumeration, not a green build.
