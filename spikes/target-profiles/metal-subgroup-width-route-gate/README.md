---
schema: "tiler-doc/v1"
id: "tiler.spike.target-profiles.metal-subgroup-width-route-gate"
kind: "experiment"
title: "The prepared subgroup-width equality gate on the real Metal route"
topics: ["target-profiles", "metal", "subgroup", "runtime", "feasibility"]
experiment_status: "blocked"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement"]
supports: ["tiler.research.target-profiles.first-macos-metal-compile-profile-authority-ledger", "tiler.research.scheduling.subgroup-execution-tier"]
entrypoints: ["spikes/target-profiles/metal-subgroup-width-route-gate/src/main.rs"]
last_verified: "2026-08-18"
ticket: "declare-metal-subgroup-realization-facts-in-the-target-profile"
---

# The prepared subgroup-width equality gate on the real Metal route

ADR 0094 decision 7 requires a subgroup-using entry's exact prepared pipeline to report the literal width the compiler verified, by equality, before the routing commit. The gate's loader half landed device-free under `carry-subgroup-width-through-exact-prepared-entry-equality`; what stayed with the owning ticket was the demonstration through the real prepared Metal path. This spike is that demonstration: real `metallib` payloads, real `MTLComputePipelineState`s, the ordinary loader route, and the governed key `tiler.target.prepared-entry.subgroup-width.v1` answered from the exact retained pipeline's `threadExecutionWidth`.

The required width is derived from `BoundMetalSubgroupDeclaration::first_m3_pro_apple9` — the evidence-backed M3 Pro subgroup declaration this spike exists to demonstrate — not from a literal written here. The run refuses, by name, any host or offline toolchain that is not the declaration's own execution row: Apple M3 Pro, macOS build `26A5388g`, offline `metalfe-32023.883`. That declaration was reachable as `tiler_build::BoundMetalSubgroupDeclaration` when this ran; it is crate-private from 2026-08-18 and **permanently** so from 2026-08-19, and the **Build exception** below states what that means for a rerun.

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

## Build exception — this harness does not compile at `main`

**Rerun this spike from commit `586c508a`**, the commit the retained log below names and the one the demonstration ran at. At the tip of `main` it does not build. Two independent causes were recorded here on 2026-08-18, neither touching what the retained run observed; the second has since been repaired and the first is permanent, so one stands.

Running `cargo check` here reports the permanent cause as a `compile_error!` in `src/main.rs` before any other diagnostic, because rustc's own message for it offers `BoundMetalCompileDeclaration` as a similar name — a different declaration that would silently supply a different width. The record is stated at the failure as well as here.

**This exception is the permanent disposition, decided 2026-08-19.** Tom accepted the host-evidence composition model ([ADR 0113](../../../docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md)); under its component 3 the M3 Pro width record stays a crate-private evidence fixture permanently, so the tip-of-`main` build path for reason 1 below is not restored — not now and not later — because the only two restoration routes were already found inadmissible in the demote delivery: a feature-gated re-export undoes the demotion, and a second copy of the rows mints a second authority over one retained measurement. `experiment_status` and the catalogue row therefore stay `blocked` as a settled state rather than a pending one. Rerunning from `586c508a` is how this evidence is reproduced, permanently. *(Reason 2 — the path-shared fixture drift — was repaired on 2026-08-19 under `keep-the-path-shared-route-gate-spike-compiling-or-make-its-breakage-loud`, as reason 2 below records. It did not and could not make this spike build at the tip, because reason 1 is permanent; what it bought is that the arrangement can no longer rot unobserved.)*

1. **The declaration is crate-private since 2026-08-18.** Tom did not accept the host-named public profile key, so `BoundMetalSubgroupDeclaration`, its error type, and `first_m3_pro_apple9` are `pub(crate)` inside `tiler-build` and the crate root no longer re-exports them. This spike is its own workspace and a separate crate, so no crate-internal driver path exists for it — `pub(crate)` is unreachable from any other crate by construction, and the alternatives (a feature-gated re-export, a second copy of the rows) would either undo the demotion or mint a second authority over the same evidence. `cargo check` here reports `error[E0432]: unresolved import tiler_build::BoundMetalSubgroupDeclaration` with `no BoundMetalSubgroupDeclaration in the root`. *(Superseded 2026-08-19. The retired sentence, kept verbatim here so the citations that name it still resolve and land on its retraction rather than on nothing: "Restoring a build path at the tip is gated on `decide-the-host-evidence-to-profile-composition-model`, which owns how single-host measured evidence composes into profile identity; until it is accepted this evidence stays crate-private." That decision is now accepted as [ADR 0113](../../../docs/decisions/0113-key-profiles-by-claim-scope-and-carry-host-evidence-as-per-row-provenance.md), and its answer is that the evidence stays crate-private permanently — so there is no gate left and no restoration pending, as the permanent-disposition paragraph above states.)*
2. **The shared runtime fixture drifted first, and independently — repaired 2026-08-19, and now checked.** `crates/tiler-runtime/tests/adapter_route/fixture.rs` is reused here through `#[path]` so there is one assembly authority rather than a copy; commit `2cb7c83c` added four `crate::adapter::ScalarEnvironmentSchema` references to it, which resolve in the runtime test binary and not in this spike's crate. That break predated the demotion — the blob at `586c508a` contains no `crate::adapter`, the blob at `2cb7c83c` contains four — so `cargo check` in this directory already failed before any visibility moved, with `error[E0433]: cannot find adapter in crate` four times.

   It was repaired under [`keep-the-path-shared-route-gate-spike-compiling-or-make-its-breakage-loud`](../../../tickets/keep-the-path-shared-route-gate-spike-compiling-or-make-its-breakage-loud.md) by declaring `ScalarEnvironmentSchema` in `fixture.rs`, where both of its constructors already lived, so the shared modules reach only each other. **The sharing arrangement's owner is `crates/tiler-runtime/tests/adapter_route`** — a change there is the shared authority moving, and consumers do not get a vote — **and its check is `crates/tiler-runtime/tests/adapter_route_portability.rs`**, a test target that compiles the shared set from a second root inside the ordinary package gate and enumerates every `#[path]` consumer in the repository. A back-edge added to a shared module is now a red gate at the moment it is written, rather than something a person discovers here months later. That check is device-free and does not depend on this spike building, which is why it survives reason 1 being permanent.

The demotion changed no row, no validation, no refusal, and no test in the declaration, and the profile key string is retained verbatim, so the retained log remains a faithful record of the code it exercised.

## Result

**Demonstration, 2026-08-18**, retained at [`results/2026-08-18-apple-m3-pro-macos27.0-26A5388g/route-gate.log`](results/2026-08-18-apple-m3-pro-macos27.0-26A5388g/route-gate.log).

Every case behaved as required: exact equality on both entries committed and the dispatched output matched the strict reference (`[15.0, 33.0]`); the mismatch, unknown-key, missing-dispatch, and constrained cross-pipeline cases each refused before the routing commit under the exact refusal class named above, quoted in the retained log; the uniform-width substitution boundary held. Both prepared pipelines reported `threadExecutionWidth = 32`, equal to the declared subject's width.
