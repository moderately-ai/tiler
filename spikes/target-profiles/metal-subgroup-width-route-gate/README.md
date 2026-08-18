---
schema: "tiler-doc/v1"
id: "tiler.spike.target-profiles.metal-subgroup-width-route-gate"
kind: "experiment"
title: "The prepared subgroup-width equality gate on the real Metal route"
topics: ["target-profiles", "metal", "subgroup", "runtime", "feasibility"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.target-profiles.first-macos-metal-compile-profile-authority-ledger", "tiler.research.scheduling.subgroup-execution-tier"]
entrypoints: ["spikes/target-profiles/metal-subgroup-width-route-gate/src/main.rs"]
last_verified: "2026-08-18"
ticket: "declare-metal-subgroup-realization-facts-in-the-target-profile"
---

# The prepared subgroup-width equality gate on the real Metal route

ADR 0094 decision 7 requires a subgroup-using entry's exact prepared pipeline to report the literal width the compiler verified, by equality, before the routing commit. The gate's loader half landed device-free under `carry-subgroup-width-through-exact-prepared-entry-equality`; what stayed with the owning ticket was the demonstration through the real prepared Metal path. This spike is that demonstration: real `metallib` payloads, real `MTLComputePipelineState`s, the ordinary loader route, and the governed key `tiler.target.prepared-entry.subgroup-width.v1` answered from the exact retained pipeline's `threadExecutionWidth`.

The required width is derived from `tiler_build::BoundMetalSubgroupDeclaration::first_m3_pro_apple9` — the evidence-backed M3 Pro subgroup declaration this spike exists to demonstrate — not from a literal written here. The run refuses, by name, any host or offline toolchain that is not the declaration's own execution row: Apple M3 Pro, macOS build `26A5388g`, offline `metalfe-32023.883`.

## What routes

A hand-assembled two-entry artifact reusing the runtime adapter-route fixture (`crates/tiler-runtime/tests/adapter_route/{fixture,image}.rs`, via `#[path]` so there is one assembly authority rather than a copy): the materialized member's real verified kernel program — a pointwise stage writing an entry-internal scratch a strict serial reduction reads, with deliberately opposite transport mappings — repackaged with `tiler.metal`/`metallib` keys over a real metallib carrying `route_pointwise_f32` and `route_reduce_f32`, compiled under the profile-strict flag vector. Every subgroup-width row is a `PreparedEntryTargetRequirement` with `ObservedEqualsRequired`, one per entry, exactly the carrier the accepted 2026-08-11 gate decision names.

**What this does not claim.** No schedule mints `ResourceRequirements::subgroup` yet, so the rows are producer-declared demonstration rows, not compiler-derived ones; the kernels execute no subgroup transfer (the packaged plan launches one thread per workgroup); and a committed route here is producer-declared equality against the fixture's profile, not host-earned eligibility. The demonstration's claim is the gate: the loader's per-entry equality against real prepared pipelines, refusing before the commit.

## Cases

1. **exact-equality-routes** — both entries require the declared width; the observer answers each request from that entry's own retained pipeline. The route commits, dispatches, and the output equals the strict host reference bit for bit.
2. **mismatch-refuses-pre-commit** — entry 1 requires 16 where the pipeline reports 32: `LoadRejection::UnsatisfiedDeferredPredicate` naming entry 1 and the observed width, before `Preflight::commit` is reachable.
3. **unknown-key-refuses-pre-commit** — a row naming `…subgroup-width.v2`: the observer's exact-match dispatch answers `Unrecognized`, and the loader refuses `UnownedPreparedEntryProperty`.
4. **missing-dispatch-refuses-pre-commit** — an adapter predating the subgroup dispatch answers every request `Unrecognized`: same pre-commit refusal class, distinct cause.
5. **cross-pipeline-substitution** — an observer answering every request from entry 0's pipeline. With entry 1 requiring 16 it refuses pre-commit on entry 1's own row; with both entries requiring 32 it commits, because **on this host every prepared pipeline reports 32** (the retained width measurement's result), so a value-level substitution is invisible here. That boundary is stated rather than hidden: the width-diverse discrimination is held by the device-free loader test `a_subgroup_width_row_is_a_per_entry_equality_and_never_a_floor` (per-entry widths 4 and 8).

The pre-commit claim is structural: every refusal returns from `resolve_target_properties`, and `commit` — which consumes the `Preflight` — is only reachable on the green path.

## Running it

From this directory, on `m3` only:

```sh
DEVELOPER_DIR=/Applications/Xcode.app cargo run -- demonstrate
```

The binary records the host and offline toolchain, the kernel and metallib digests, the declaration's stated subject, every prepared-entry request the loader issued with its answer, and each case's verdict with the refusal quoted. On any other host it refuses before preparing a pipeline; the refusal on the coordination M4 Max (`26A5406e`) was watched firing.

The reused fixture modules produce dead-code warnings for the members this spike does not route; they are expected.

No `make` target reaches here, per [`spikes/README.md`](../../README.md).

## Result

**Demonstration, 2026-08-18**, retained at [`results/2026-08-18-apple-m3-pro-macos27.0-26A5388g/route-gate.log`](results/2026-08-18-apple-m3-pro-macos27.0-26A5388g/route-gate.log).

Every case behaved as required: exact equality on both entries committed and the dispatched output matched the strict reference (`[15.0, 33.0]`); the mismatch, unknown-key, missing-dispatch, and constrained cross-pipeline cases each refused before the routing commit under the exact refusal class named above, quoted in the retained log; the uniform-width substitution boundary held. Both prepared pipelines reported `threadExecutionWidth = 32`, equal to the declared subject's width.
