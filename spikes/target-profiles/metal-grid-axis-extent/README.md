---
schema: "tiler-doc/v1"
id: "tiler.spike.target-profiles.metal-grid-axis-extent"
kind: "experiment"
title: "What grid-axis extent this Apple9 macOS row actually dispatches"
topics: ["target-profiles", "metal", "apple-targets", "feasibility", "provenance"]
experiment_status: "reproducible"
implementation_status: "spike-only"
evidence_classes: ["bounded-measurement", "exhaustive-finite"]
supports: ["tiler.research.target-profiles.first-macos-metal-compile-profile-authority-ledger"]
entrypoints: ["spikes/target-profiles/metal-grid-axis-extent/src/main.rs"]
last_verified: "2026-08-18"
ticket: "establish-an-upper-bound-authority-for-the-metal-grid-axis-row"
---

# What grid-axis extent this Apple9 macOS row actually dispatches

This spike supplies the number the authoritative macOS Metal profile's `GridAxisThreads` row had no authority for. It exists because of an asymmetry that is easy to miss and decides the whole question: **the row is consumed as a guarantee, so it needs a lower bound on capability, and every normative source available is an upper bound on the space.**

`CapabilityAxis::GridAxisThreads = N` is read by physical feasibility as *every dispatch with axis extent at most N is admissible*. An authority that says "nothing above M is expressible" forbids declaring more than M; it licenses no value at all. So the eliminations below are not a failure to find the source — they are the finding that the normative sources are the wrong shape for this row, and that a bounded measurement is the only class that can supply it.

## What the normative routes establish, and why none of them sources the row

**Fact — the Metal Feature Set Tables state no compute-grid maximum.** The vendored [2025-10-20 tables](../../../docs/research/apple-targets/sources/apple-metal-feature-set-tables-2025-10-20.pdf) carry exactly two grid rows, `Maximum threadgroups per object shader grid` and `Maximum threadgroups per mesh shader grid`. Neither is a compute-grid capacity, and the `Apple9` column of the second reads 1,024 — a mesh-shader figure a reader could easily misread as one. There is no compute row to cite.

**Fact — the SDK header bounds nothing, in either installed SDK.** `MTLComputeCommandEncoder.h` documents `dispatchThreads:threadsPerThreadgroup:` as "Enqueue a compute function dispatch using an arbitrarily-sized grid", and its `@discussion` scopes that phrase precisely: "threadsPerGrid does not have to be a multiple of the  threadGroup size". That is a statement about divisibility, not magnitude. `MTLTypes.h` types every `MTLSize` dimension as `NSUInteger`. Together they prove any extent is *representable* — which is what the superseded row cited, and it is why that row's own comment recorded that four was chosen to cover a program rather than derived from anything.

The macOS 26.5 and 27.0 SDK headers agree, and the check is a byte comparison rather than a reading: `MTLComputeCommandEncoder.h` is identical in both (SHA-256 `610bcf8f3e6cb6a7067622f4395d8aa292c56226afde457ac6cb902937872b7b`), and `MTLTypes.h` differs by exactly one added blank line at line 106, with the `MTLSize` definition byte-identical (SHA-256 `dbb86ed168a92c8a52464b93b057af3d3e513acae82fe40b92d70d5c370d1104` over lines 25–37 of each).

**Fact — the one numeric bound in the header is on a route Tiler does not encode.** `MTLDispatchThreadsIndirectArguments` (`MTLComputeCommandEncoder.h:31-34`, both SDKs) types its grid as `uint32_t threadsPerGrid[3]`, so no extent above `2^32 - 1` is expressible in an *indirect* dispatch argument buffer. Tiler encodes the direct route — `encoder.dispatch_threads(MTLSize::new(grid_threads, 1, 1), ...)` in `prototypes/serial-sum-run/src/proof.rs` — whose `MTLSize` is `NSUInteger`. Citing the indirect struct for the direct route would be citing a bound that does not apply to it.

**Fact — MSL caps the addressable grid at `2^32`, and that is a ceiling rather than a source.** MSL 4.0 §5.2.3.6 Table 5.8 lists the corresponding data types for `thread_position_in_grid` as `ushort, ushort2, ushort3, uint, uint2, or uint3`, and offers nothing wider; `threads_per_grid` carries the same list, and the notes below the table require the two to match. The 4.1 specification (2026-06-04) is unchanged on this row. No kernel in this language can therefore distinguish more than `2^32` positions along one axis, whatever the API represents. This bounds what may ever be declared — it does not say any particular extent works, so it cannot fill the row.

## What this measures

One invocation per grid point writes `tid ^ salt` into its own slot of a buffer poisoned with `0xDEADBEEF` before every dispatch. Two things follow, and both are what the fault-injection below proves rather than asserts: an invocation that did not run leaves the poison, and the salt arrives in a device buffer at dispatch time, so no fill the host could have performed reproduces the expected pattern. **Every slot is compared, never sampled** — the claim is about the whole grid, so a sampled check would not support it.

Three things are held to the profile's own choices rather than to whatever is convenient:

- the kernel is compiled **offline through the production `tiler_metal_aot::CompileRequest`** — target `air64-apple-macos26.0`, `-std=metal4.0`, `OptimizationLevel::Default`, `NumericalRealization::strict_baseline()` — so the executed compilation selection equals the one every production plan compiles with, byte for byte, and the record's `selection.compile_flags` row is transcribed from the executed request's own provenance (offline rather than through `newLibraryWithSource:` because [ADR 0086](../../../docs/decisions/0086-require-attributable-or-attested-native-translation.md) item 4 excludes the runtime compiler by name; the 2026-08-04 run predated this rework and compiled through a hand-spelled `xcrun` invocation carrying target and standard only);
- it declares `uint tid [[thread_position_in_grid]]`, which is the launch-index realization the profile selects (`LaunchIndexRealization::ThreadPositionInGridUInt`);
- it is dispatched through `dispatchThreads:threadsPerThreadgroup:` with an `MTLSize`, which is the route the runtime prototype encodes.

Every extent runs at three threadgroup widths — 1, the pipeline's reported `threadExecutionWidth`, and its reported `maxTotalThreadsPerThreadgroup` — all read from the prepared pipeline rather than from the feature tables, on the same grounds the ledger's workgroup row is a deferred query. Width 1 is what every current Tiler independent-invocation region declares; the other two make the **tail case** reachable, where the grid extent is not a multiple of the threadgroup width. That case is the one place a wide grid could plausibly misbehave while a narrow one does not, and `dispatchThreads:` is exactly the entry point that admits it.

## Running it

```sh
cd spikes/target-profiles/metal-grid-axis-extent
DEVELOPER_DIR=/Applications/Xcode.app cargo run --release > results/<date>-<host>/extent.tsv
```

`DEVELOPER_DIR` selects the offline toolchain **for this invocation only** and mutates no host state; it is how the run reaches the exact Xcode 26.6 / SDK 26.5 toolchain the ledger records while a newer Xcode is the default selection. The harness records whatever toolchain answered rather than assuming one, so a run under a different toolchain is self-describing instead of silently mislabelled.

No `make` target reaches here, per [`spikes/README.md`](../../README.md).

## Result — 2026-08-18 re-measurement through the production request

**Measurement, 2026-08-18**, retained at [`results/2026-08-18-apple-m4-max-macos27.0-26A5406e/`](results/2026-08-18-apple-m4-max-macos27.0-26A5406e/extent.tsv), run under the accepted (R, R) disposition of `resolve-the-retained-metal-profile-measurement-invocation-authority`.

The probe was compiled through the production `tiler_metal_aot::CompileRequest`, and the retained header's `selection.compile_flags` row — transcribed by the harness from the executed compilation's provenance — reads exactly `-target air64-apple-macos26.0 -std=metal4.0 -O2 -fmetal-math-mode=safe -fmetal-math-fp32-functions=precise -ffp-contract=off`, with zero additional linker flags after `xcrun --sdk macosx --find metallib` tool selection (`selection.link_flag_count 0`). That is byte-for-byte the selection every production plan compiles with, so the selection identity derives from the request whose compilation executed.

Offline compilation environment: `Apple metal version 32023.883 (metalfe-32023.883)`, `AIR-LLD 32023.883 (metalfe-32023.883)`, Xcode 26.6 build 17F113, macOS SDK 26.5 build 25F70 — byte-identical to the authority ledger's offline table. Execution environment: macOS 27.0 build **`26A5406e`**, `arm64`, Apple M4 Max, `device_apple9_support` true — a **different OS build** from the 2026-08-04 run's `26A5388g`, declared truthfully in this record's own environment rows; the sibling [`environment.tsv`](results/2026-08-18-apple-m4-max-macos27.0-26A5406e/environment.tsv) pins the harness source SHA-256s, the repository base `ea5c615db289e4fee044045ec948c5eecca68ffe`, and the result digest, closing the harness-binding gap the 2026-08-04 record carried.

**Every one of the 6,294 dispatched rows reached `Completed` and verified every slot.** The widest extent verified at all three threadgroup widths is **268,435,456** (`2^28`) — the same ladder, the same stop condition, and the same outcome as the 2026-08-04 run. No rung failed, so the row is not narrowed. The prepared pipeline reported `maxTotalThreadsPerThreadgroup = 1024` and `threadExecutionWidth = 32`; the device reported `maxBufferLength = 22,613,000,192`.

Both mutation proofs were rerun under the new compile path before the result was trusted and are retained in the same directory's [`perturbations.txt`](results/2026-08-18-apple-m4-max-macos27.0-26A5406e/perturbations.txt): the dropped salt failed every width's first rung with `observed 00000000`, and the withheld third invocation failed extent 3 with `observed deadbeef` at every width.

## Result — 2026-08-04 (retained; pre-rework compilation)

**Measurement, 2026-08-04**, retained at [`results/2026-08-04-apple-m4-max-macos27.0-26A5388g/extent.tsv`](results/2026-08-04-apple-m4-max-macos27.0-26A5388g/extent.tsv). This run predates the production-`CompileRequest` rework: its probe was compiled by a hand-spelled `xcrun` invocation carrying target and standard only, with no optimization or numerical-selection flags, and its record binds no harness hash or repository revision. Its disposition under the accepted (R, R) decision is recorded in `resolve-the-retained-metal-profile-measurement-invocation-authority`.

Offline compilation environment: `Apple metal version 32023.883 (metalfe-32023.883)`, `AIR-LLD 32023.883 (metalfe-32023.883)`, Xcode 26.6 build 17F113, macOS SDK 26.5 build 25F70, `-std=metal4.0`, target `air64-apple-macos26.0`. Execution environment: macOS 27.0 build `26A5388g`, `arm64`, Apple M4 Max, `device_apple9_support` true — the ledger's execution environment B, which the tree-width policy and every dispatchability and numerical row are still scoped to. *(Corrected 2026-08-19 at the grid-and-cost reseat: this sentence read "**Both match the authority ledger's two environment tables in every field**, which is why the row this measurement sources joins the profile's existing measurement context instead of adding a second one." Neither half stands. The ledger now presents one shared offline table and two execution tables, one per environment; and **this record sources nothing** — the grid-axis row was reseated onto the 2026-08-18 re-measurement above, so this run is retained evidence of what the row rested on between 2026-08-04 and then.)*

**Every one of the 6,294 dispatched rows reached `Completed` and verified every slot.** The widest extent verified at all three threadgroup widths is **268,435,456** (`2^28`). Evidence below 2,049 is exhaustive over the integers — every extent, not a ladder — and above it the run samples each power of two from `2^12` with both of its neighbours.

The prepared pipeline reported `maxTotalThreadsPerThreadgroup = 1024` and `threadExecutionWidth = 32`; the device reported `maxBufferLength = 22,613,000,192`.

**`2^28` is the experiment's stop condition, not an observed limit.** Every rung passed, so nothing here says `2^28 + 1` would fail. The ceiling is set by two things stated rather than hidden: complete verification costs four bytes of device memory per thread, so the top rung is a 1 GiB buffer; and `2^28` sits sixteen times below the MSL language ceiling while covering the widest single tensor in the corpus this project measures against — Qwen3-0.6B's `151,936 x 1,024` embedding matrix, about `2^27.2` elements — so the row it sources does not refuse a plan for a tensor the project already handles.

## Every check was proved able to fail

Two perturbations, each run before the result above was trusted, each restored afterwards:

- **The salt dropped.** With the kernel writing `out[tid] = tid`, every rung reported `first_mismatch 0` with `observed 00000000` against the expected `9e3779b9`. The salt is therefore load-bearing: the check cannot be satisfied by a value the kernel could produce without reading the dispatch-time buffer.
- **Every third invocation withholding its write.** With the kernel guarded by `if (tid % 3u != 2u)`, extent 3 reported `verified_slots 2`, `first_mismatch 2`, `observed deadbeef`. Poison detection therefore distinguishes "the invocation did not run" from "the invocation ran", which is what makes a passing row evidence that the whole grid executed.

Without these, a harness that read a stale buffer, or one whose verification loop never executed, would be indistinguishable from a passing run.

## Boundary

- **One environment per retained record, and the row it sources says so.** The declared profile row carries `TargetCompileProfileMeasurementSource`, whose validity is the exact offline and execution environments of the one retained record it consumes, together. The two retained runs above were taken on two different OS builds and never share a context. It is not a portable guarantee, not an Apple-family claim, and not a statement about any other OS row, GPU family, or toolchain.
- **It establishes a floor, never a maximum.** Nothing here measures a failure, so nothing here says where one is. A later run that found a failing extent would narrow the row; this one cannot widen into "the hardware supports exactly this much".
- **Exhaustive below 2,049 and sampled above.** The guarantee between two sampled rungs is an interpolation over a monotone-looking observation, and the retained TSV is what a reader checks that against.
- **No performance claim of any kind.** Nothing is timed. The result is about which dispatches execute correctly, not how fast any of them runs.
- **One kernel shape.** A single buffer write per invocation. It says nothing about extents reachable by a kernel with different resource requirements, and the profile's other rows — buffer bindings, local memory, the prepared workgroup query — remain the separate constraints they were.

## What consumes it

`crates/tiler-build/src/metal_declaration.rs`'s `FIRST_MACOS_APPLE9` declares `grid_axis_threads` from the **2026-08-18** run through the `grid-axis` population's own measured source, replacing a normatively sourced four whose authority licensed no number. The [authority ledger](../../../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md) records the row, its class, and what it does not cover.

Widening the row moves the profile's canonical descriptor and therefore every identity derived from it; the ledger enumerates the pins that moved with it.

[`spikes/program-planning/reduction-crossover`](../../program-planning/reduction-crossover/README.md) is the sweep whose one-shape result this unblocks.
