---
id: produce-the-conformance-envelope-in-process-so-the-routed-half-reaches-the-gate
title: Produce the conformance envelope in process so the routed half reaches the gate
status: todo
priority: p1
dependencies: []
related: [carry-the-device-executed-value-proof-into-the-conformance-crate]
scopes: [implementation/conformance]
shared_scopes: [project/tickets]
paths: []
tags: [conformance, coverage]
---
## The gap

[`carry-the-device-executed-value-proof-into-the-conformance-crate`](carry-the-device-executed-value-proof-into-the-conformance-crate.md) moved the device-executed value proof into `crates/tiler-conformance` and every claim is re-proved. **But the envelope half is gated on `TILER_CONFORMANCE_ARTIFACT_BASE`**, so in `make full` it reports the artifact boundary unavailable — naming `cargo run -p tiler-prototype-compile -- --out <base>` — and only the device-free half runs.

So the routed leg is *reachable* rather than *reached*. That is a strictly better position than the prototype it came from, where nothing ran under the gate at all, and it is not the outcome the migration existed for.

**The migrating worker named this as the single highest-value follow-up and did not take it**, because the ticket that authorized the migration scoped it out — correctly, since taking it would have been outcome expansion mid-migration.

## Why it is cheap

**No new dependency is needed.** `crates/tiler-conformance` already declares the producer's entire row — artifact, build, compiler, IR, metal, metal-aot, reference, runtime — because the migration's normal-dependency decision gave it the whole vertical. What is missing is the *call*, not the reach.

## What this owes

- The envelope produced in process, so the routed half runs whenever the device half can and the environment variable stops being the gate on coverage.
- **The unavailable path preserved, and still watched both ways.** A host without the toolchain must still report the measurement boundary unavailable and pass; under `TILER_REQUIRE_METAL_CONFORMANCE=1` it must still fail loudly. Producing in process must not turn an unavailable environment into a hard failure on an ordinary host — that would trade one coverage gap for a broken gate.
- The environment variable either retired or given a stated remaining purpose. A knob that no longer gates anything is the stale-disclosure pattern this repository keeps finding.
- Whatever the in-process producer costs in gate wall-clock, measured and reported. `make full` already carries a 71-second decoder-layer test and the repository tracks its own critical path; adding an offline Metal compile to every run is a real cost that should be stated rather than discovered.

## Explicit non-goals

Do not widen what the routed half *proves* — the claims are already enumerated and passing; this is about when they run. Do not delete `prototypes/serial-sum-run`: its retirement is a separate fork that is Tom's, and its remaining unique value is a loader fixture that could not move (see below).

## The related thing that could not move, recorded so it is not re-attempted blindly

The prototype's `#[cfg(test)]` loader fixture **cannot** go to `crates/tiler-runtime/tests/`. It compiles through `tiler_compiler::session::compile` and reaches `tiler_build::realization::translate`, `tiler_metal::applicability` and `metal`; `identity_join`'s `the_consumer_links_no_compiler_emitter_or_build_provider` reads `Cargo.lock` — which merges normal, build *and* development edges — and asserts the loader's closure contains none of those packages. The move needs four of the five forbidden packages as dev-dependencies and turns that test red. A compiler-free rewrite onto `adapter_route/fixture.rs`'s existing assembler is possible and is its own ticket.

## Closes when

The routed half runs in `make full` on a qualified host without an environment variable, the unavailable path is preserved and watched both ways, the variable is retired or re-justified, and the gate's added wall-clock is measured.
