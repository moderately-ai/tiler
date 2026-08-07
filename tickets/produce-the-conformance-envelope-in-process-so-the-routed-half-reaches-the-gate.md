---
id: produce-the-conformance-envelope-in-process-so-the-routed-half-reaches-the-gate
title: Produce the conformance envelope in process so the routed half reaches the gate
status: done
priority: p1
dependencies: []
related: [carry-the-device-executed-value-proof-into-the-conformance-crate]
scopes: [implementation/conformance, implementation/cargo-lock]
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

## Outcome

`crates/tiler-conformance/src/publication.rs` publishes the eight members the routed half opens — six serial-sum members (three reduction classes times the fused and materialized roles) and the two contraction members — into a private temporary directory a `Drop` guard removes, and the routed runs open them from there. Nothing is gated on an ambient input any more. On this host `make full` now executes 30 serial-sum operand cases across six members plus five adversarial contraction cases and the `w_decode_kv` cell, including the eight fail-closed loader probes and the six injected device-preflight refusals, all of which previously reported the artifact boundary unavailable on every gate run.

**No backend fact was restated to get there.** Publishing goes through `tiler_build::accept_or_publish_metal_plan`, the same public path `prototypes/serial-sum-compile` uses, so the members cross real MSL emission, `metal`, `metallib`, neutral artifact assembly, and expansion-cache acceptance. That is the one thing the ticket's "no new dependency is needed" claim got wrong: `accept_or_publish_metal_plan` takes an `&ExpansionCache`, so `tiler-cache` is now a direct edge (`Cargo.lock` gains exactly one line, and `implementation/cargo-lock` was added to this ticket's scopes). The alternative — assembling through `assemble_plan_artifact` directly — would have required this crate to restate the Metal backend's own per-stage binding kinds, zero-work dispatch policy, and launch preconditions, which is a backend fact and not a conformance one; the envelope would then have been one this workspace does not actually ship.

**The environment variable is retired, not kept as an override.** `TILER_CONFORMANCE_ARTIFACT_BASE`, `PRODUCER_COMMAND`, `PublishedBase`, `published_base`, and `require_or_report_base` are gone, and with them `an_absent_published_base_names_the_producer_command`. An override that forced a prebuilt base was considered and rejected: nothing in `make full` would set it, so it would be a second unexercised path rather than a capability, and the cross-executable agreement it could carry still lives where it always did, in the `prototypes/serial-sum-compile` / `prototypes/serial-sum-run` pair. There is now **one** boundary rather than two, because publishing needs exactly the toolchain routing needs.

**The unavailable path is preserved and was watched both ways**, against the compiled test binary with `PATH` emptied so `xcrun` cannot be found:

```text
env -i PATH=/var/empty … ./target/debug/deps/tiler_conformance-… --test-threads=1 --nocapture envelope::tests
envelope matrix: MEASUREMENT BOUNDARY UNAVAILABLE — no qualified Apple Metal toolchain resolved:
  Apple Metal toolchain unavailable (metal, discovery): could not run xcrun: No such file or directory (os error 2).
  The deterministic half above ran; nothing here claims a device result.
test result: ok. 12 passed; 0 failed; …          (exit 0)
```

```text
env -i PATH=/var/empty … TILER_REQUIRE_METAL_CONFORMANCE=1 ./target/debug/deps/tiler_conformance-… envelope::tests
panicked at crates/tiler-conformance/src/measurement.rs:164:13:
TILER_REQUIRE_METAL_CONFORMANCE is set and the measured half is unavailable: no qualified Apple Metal
  toolchain resolved: … could not run xcrun: No such file or directory (os error 2)
test result: FAILED. 10 passed; 2 failed; …      (exit 101)
```

**Added wall-clock, measured on the idle M4 Max / macOS 27.0 `26A5388g` host.** Three runs each, dev profile, warm:

| measurement | with the routed pair | without it | delta |
| --- | --- | --- | --- |
| `cargo nextest run -p tiler-conformance --test-threads=1` | 2.56 / 2.61 / 2.63 s | 1.70 / 1.70 / 1.70 s | **+0.90 s** |
| `cargo nextest run -p tiler-conformance` (default parallelism) | 0.77 / 0.75 / 0.78 s | 0.45 / 0.44 / 0.48 s | **+0.31 s** |
| `cargo nextest run --workspace` (what `make full` runs) | 29.97 / 29.37 s | 29.37 / 29.16 s | **+0.4 s** |

So the gate cost is **about four tenths of a second**, against a 71-second decoder-layer test and an 87-second `make full`. Publishing is the larger half of the 0.90 s serial figure: routing alone cost 0.33 s when the members were pre-published. **The one number that looks alarming is a cold-start artefact and is recorded so nobody re-derives it:** the first `cargo run -p tiler-prototype-compile -- --out …` on this host took **14.4 s** at 4% CPU, and the identical command took **0.68 s** immediately afterwards. That 14 s is one-time `xcrun` warm-up, not per-member work; a host whose first `make full` of the day pays it will see it once.

**Deliberately not converged: `PUBLISHED_ROWS` is 1 and `serial_sum::ROWS` is 4.** The routed half derives the program it compares identities against from the artifact's *declared* shape, and a run substituting its own row count would be invisible if the two agreed — the defect this vertical actually suffered for a month. `publication::tests::the_published_rows_are_not_the_direct_paths_own` holds them apart.

**What the in-process move gives up, stated rather than absorbed.** Producer and consumer are one process from one tree now, so agreement between two independently maintained halves is no longer observable here; it never was observable in the gate, since the ambient input was unset. The stale sentences that claimed otherwise were corrected rather than left: `envelope`'s header ("an artifact a separate producer wrote"), `contraction_structure`'s renaming-invariance claim ("the two processes reach the same canonical encoding"), `ForeignProgram`'s "the producer and this consumer have drifted", `read_artifact`'s identity note, and the contraction recognizer's "a recognizer that only ever saw the artifact its own producer writes" — whose premise is now the actual state, which makes its negatives more necessary rather than less.

**Evidence.** `make full` green at the reported commit in **49.06 s** with the release artifacts warm, and green in **1:26.93** on the run before it, which recompiled `tiler-reference` and `tiler-compiler` in release (25.42 s of the difference). `cargo nextest run --workspace` 3,034 passed / 7 skipped; `cargo clippy -p tiler-conformance --all-targets -- -D warnings`, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-conformance`, `git diff --check`, and `tkt lint` all clean; `tkt guard` reports affected scopes equal to declared scopes, verdict WARN from shared `project/tickets` overlaps and three direct overlaps with `done` tickets, exit 0. The unsafe population is unchanged at **two sites in `device_buffer.rs`** with no crate-level allow (`the_unsafe_site_population_is_the_two_named_ones` passes; the new modules carry none). No identity pin moved: `crates/tiler-build/src/metal_plan.rs` still holds `ARTIFACT_IDENTITY = 7a2bfe51619c05a13fe86cd973e1dfa85c7353da33e4e75af0531068b774357d`, `CACHE_SUBJECT = 8bdcde644d7df6d4ca95736f445a011b2d163efdfb3ba93a5c0a954d139b1aa2`, and `FIXED_CONTENT_BYTES = 65_294`, untouched and passing.

**One thing found and not fixed here.** `cargo clippy -p tiler-conformance --all-targets --target x86_64-unknown-linux-gnu -- -D warnings` fails, and it already failed at this ticket's base: 32 of its 57 dead-code errors name symbols in `envelope.rs` and `serial_sum.rs` this branch did not touch (`SOLE_DELIVERY`, `bind_declared_interface`, every `probe_*`), which are reachable only from the `cfg(target_os = "macos")` modules. The `conform-the-bf16-vertical-end-to-end` claim that the non-Apple branch compiles clean predates the envelope module and is stale. This branch adds ~25 more of the same class in `publication`. The check is not part of `make full`, so nothing regressed in the gate — but the crate's "a non-Apple host runs the device-free half" claim is unverified for everything except the bf16 vertical, and that wants its own ticket.

## Outcome — delivered 2026-08-07 at `082ad4b9`

**The routed half now runs in `make full`.** `crates/tiler-conformance/src/publication.rs` publishes the eight members into a private temp directory a `Drop` guard removes, and the routed runs open them there. The gate now executes 30 serial-sum operand cases across six members, five adversarial contraction cases, the `w_decode_kv` cell whose executed bytes carry the retained realization-probe SHA-256, eight fail-closed loader probes and six injected device-preflight refusals — **all of which previously reported the artifact boundary unavailable on every gate run.**

Publishing goes through `tiler_build::accept_or_publish_metal_plan`, the same public path the prototype producer uses, so members cross real MSL emission, `metal`/`metallib`, neutral artifact assembly and expansion-cache acceptance.

**The ticket's "no new dependency is needed" was wrong, and the worker said so.** `accept_or_publish_metal_plan` takes an `&ExpansionCache`, so `tiler-cache` became a direct edge and `implementation/cargo-lock` was added to the scopes. The alternative — calling `assemble_plan_artifact` directly — would have forced this crate to restate the Metal backend's binding kinds, zero-work dispatch policy and launch preconditions, i.e. publish an envelope the workspace does not ship. That is the right reason to accept the edge rather than route around it.

**The environment variable was retired rather than kept as an override**, on the ground that an override nothing in `make full` sets is a second unexercised path, and the cross-executable agreement it could have carried still lives in the untouched prototype pair. One boundary now, not two.

**The unavailable path is watched both ways with real output** — passing with `PATH` emptied so `xcrun` is absent, failing loudly under `TILER_REQUIRE_METAL_CONFORMANCE=1`. So producing in process did not trade a coverage gap for a broken gate, which was the requirement most at risk.

**Measured added wall-clock: ~0.4 s** on the workspace run, against an 87-second gate carrying a 71-second decoder test. Recorded rather than estimated, with a note that the first `cargo run -p tiler-prototype-compile` on this host took 14.4 s at 4% CPU and the identical command immediately after took 0.68 s — one-time `xcrun` warm-up, not per-member work, so nobody re-derives it.

**Unsafe: still exactly two sites**, both in `device_buffer.rs`, no crate-level allow, and the new modules contain none. **Pins unmoved** — `git diff` against `crates/tiler-build/` is empty. Test count 47 → 52. `make full` green at 49 s.

**Stale prose corrected rather than left**, which producing in process made necessary: five sentences describing "a separate producer", "the two processes", and producer/consumer drift are no longer true. The sharpest is the contraction recognizer's note that "a recognizer that only ever saw the artifact its own producer writes would accept anything" — whose premise is now the actual state, making its negative checks *more* necessary rather than less.

**Filed rather than fixed:** [`restore-the-conformance-crates-non-apple-build-and-lint-claim`](restore-the-conformance-crates-non-apple-build-and-lint-claim.md). Clippy for a non-Apple target fails — **and already failed at this branch's base**, with 32 of 57 dead-code errors naming untouched symbols. The crate's "a non-Apple host runs the device-free half" claim is therefore unverified beyond the bf16 vertical. The check is not in `make full`, so nothing regressed; choosing between per-item `cfg_attr` allows, per-module gates, or gating the whole envelope route (which would *shrink* non-Apple coverage) is a decision that wants its own ticket rather than a reflex.
