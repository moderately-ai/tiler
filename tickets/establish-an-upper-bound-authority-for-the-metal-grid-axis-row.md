---
id: establish-an-upper-bound-authority-for-the-metal-grid-axis-row
title: Establish an upper-bound authority for the Metal grid-axis row
status: in-progress
priority: p1
dependencies: []
related: [calibrate-and-activate-parallel-reduction-selection]
scopes: [research/target-profiles, implementation/build, implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [research, target-profiles, measurement]
claimed_from: todo
assignee: agent-grid-axis
lease_expires_at: 1785879207
---
## User-visible outcome

The authoritative macOS Metal profile declares a grid-axis bound established by a real upper-bound authority rather than by a conservative representability floor, so programs wider than four elements can compile and the reduction strategies become comparable on more than one shape.

## Why this exists

**Measurement, 2026-08-02 — the current row is a floor, and it collapses the parallel-reduction measurable domain to a single point.** [The retained sweep](../spikes/program-planning/reduction-crossover/README.md) compiled a reduction program family across 36 shapes against `tiler.metal.macos-apple9.msl4-0.f32.v1` under `NumericalContract::FLUSH_AND_REASSOCIATE_F32`, on a host matching the ledger's execution-environment row in every field. Exactly one shape retains all three reduction strategies at once: one row of four contributors. Every wider shape is refused by hard feasibility with `event=feasibility:grid-axis:rejected:target-infeasible:threads=<required>:4` at the pointwise prologue.

**Fact — the row says of itself that it is not a maximum.** `crates/tiler-build/src/metal_declaration.rs:185-188` declares `grid_axis_threads: 4` with the comment: the macOS 26.5 SDK's `dispatchThreads:` contract "proves extent 4 is representable and establishes no upper bound at all, so 4 is a deliberately conservative compile guarantee rather than a maximum." The compiler-side governed profile carries the same four with the same reasoning (`crates/tiler-compiler/src/target.rs`, `TargetProfileBuilder::governed`), citing `MTLComputeCommandEncoder.h` and `MTLTypes.h` as proving representability and explicitly not proving 65,535, an Apple-family maximum, or any prepared pipeline's capacity.

**Inference — this is an absent authority, not a hardware limit.** No measurement would confirm four; the row is what the primary sources happen to prove, and they prove a lower bound. So raising it is a question of finding the authority that states a real maximum, at the right phase, rather than of running a probe against the current one.

## What blocks on it

- [`calibrate-and-activate-parallel-reduction-selection`](calibrate-and-activate-parallel-reduction-selection.md) cannot establish a crossover: a crossover needs at least two shapes on which the strategies coexist, and the domain is one point. The single point is forced by arithmetic — `governed_partition` withholds both parallel strategies below four contributors and the grid axis caps `rows * contributors`, so `4 <= contributors <= rows * contributors <= bound`.
- `target::tests::only_one_shape_admits_all_three_reduction_strategies` fails when this row widens, which is the designed signal that calibration has become possible.

## Implementation keys

Establish the authority first and the number second. Candidate authorities, each to be accepted or eliminated with the ground stated: an Apple feature-table row stating a maximum grid size per dimension; an SDK header or specification sentence bounding `MTLSize` dimensions for `dispatchThreads:`; or a retained device measurement, which qualifies a bounded profile rather than a portable guarantee and must be declared through the measured source with its execution environment attached.

Do not raise the number without moving the authority with it. A widened bound carrying the existing representability citation would be the citation saying something it does not say, which is exactly what the authority ledger exists to prevent.

Identity moves when the row moves: the profile's canonical descriptor is encoded into artifact identity and the cache subject, so every pinned identity must be recomputed on the tree the change lands into, with each moved pin enumerated.

## Closes when

The grid-axis row cites an authority that states an upper bound, the ledger records which class that authority is and what it does not cover, the descriptor and every pinned identity move together in one commit, and the reduction-domain trigger test reports the new domain. Reruning the retained sweep records what became measurable.

## Outcome — 2026-08-04: the row is measured, because no normative source can fill it

### The finding is an asymmetry, not a search result

**Inference, and it is the whole ticket.** `CapabilityAxis::GridAxisThreads = N` is consumed by physical feasibility as a **guarantee**: a plan is admissible when its required extent is no greater than `N`. Its authority therefore has to state a **floor on capability** — *extents up to N work*. Every normative source about Metal grid extent states a **ceiling on the space**, and a ceiling forbids declaring more without licensing anything inside it. The ticket asked for "an upper-bound authority"; what the row actually needs is a lower bound, and that is why the eliminations below are not a failure to find the source but the reason a measurement is the only admissible class.

### Each candidate authority, accepted or eliminated

1. **An Apple feature-table row — eliminated, verified rather than assumed.** The vendored 2025-10-20 tables carry exactly two grid rows: `Maximum threadgroups per object shader grid` and `Maximum threadgroups per mesh shader grid`. Neither is a compute-grid capacity. The `Apple9` column of the second reads 1,024, a mesh-shader figure easily misread as a compute limit.
2. **An SDK sentence bounding `MTLSize` for `dispatchThreads:` — eliminated, in both installed SDKs.** `MTLComputeCommandEncoder.h` says "Enqueue a compute function dispatch using an arbitrarily-sized grid", and its own `@discussion` scopes that phrase to divisibility: "threadsPerGrid does not have to be a multiple of the  threadGroup size". `MTLTypes.h` types every `MTLSize` dimension as `NSUInteger`. **The 26.5 and 27.0 SDKs agree by byte comparison:** `MTLComputeCommandEncoder.h` is identical in both (SHA-256 `610bcf8f3e6cb6a7067622f4395d8aa292c56226afde457ac6cb902937872b7b`); `MTLTypes.h` differs by one added blank line at line 106 with the `MTLSize` definition byte-identical (SHA-256 `dbb86ed168a92c8a52464b93b057af3d3e513acae82fe40b92d70d5c370d1104` over lines 25–37 of each). The Xcode 27.0 beta build is `27A5228h` (the dispatch brief said `27A5228f`; corrected here from `xcodebuild -version`).
3. **The one numeric bound the header does carry — eliminated as inapplicable.** `MTLDispatchThreadsIndirectArguments` types its grid as `uint32_t threadsPerGrid[3]`, bounding the *indirect* route at `2^32 - 1`. Tiler encodes the direct route (`encoder.dispatch_threads(MTLSize::new(...), ...)`), whose `MTLSize` is `NSUInteger`. Reconsideration trigger: an indirect dispatch route reaching this profile.
4. **A genuine normative ceiling, recorded but not used as the source.** MSL 4.0 §5.2.3.6 Table 5.8 types `thread_position_in_grid` as `ushort, ushort2, ushort3, uint, uint2, or uint3` and nothing wider; MSL 4.1 (2026-06-04) is unchanged. No kernel in this language can distinguish more than `2^32` positions on an axis, so a bound above that would be a guarantee the emission cannot keep. This caps the row forever; it licenses no value.
5. **A retained device measurement — accepted.** [`spikes/target-profiles/metal-grid-axis-extent`](../spikes/target-profiles/metal-grid-axis-extent/README.md).

### The measurement

**Measurement, 2026-08-04**, retained at `spikes/target-profiles/metal-grid-axis-extent/results/2026-08-04-apple-m4-max-macos27.0-26A5388g/extent.tsv`. The ladder dispatched through this profile's own compilation (`-std=metal4.0`, `air64-apple-macos26.0`, offline), its own launch realization (`uint tid [[thread_position_in_grid]]`), and its own dispatch route, verifying **every slot** of a poisoned buffer at threadgroup widths 1, 32, and 1,024. All **6,294** dispatched rows reached `Completed` and verified. Widest extent verified at every width: **268,435,456** (`2^28`). Exhaustive over the integers below 2,049; sampled at each power of two from `2^12` with both neighbours above it.

**The environment is the ledger's own, in every field** — offline `Apple metal version 32023.883`, `AIR-LLD 32023.883`, Xcode 26.6 (17F113), SDK 26.5 (25F70); execution macOS 27.0 `26A5388g`, `arm64`, Apple M4 Max. It was reached with `DEVELOPER_DIR=/Applications/Xcode.app` for the invocation only, which mutates nothing, while a newer Xcode is the default selection. That is why the row **shares** the profile's existing measurement source rather than adding a second context.

**Both checks were proved able to fail before the result was trusted.** Dropping the salt made every rung report `first_mismatch 0, observed 00000000`; withholding every third invocation's write made extent 3 report `verified_slots 2, first_mismatch 2, observed deadbeef`.

**`2^28` is the run's stop condition, not an observed limit.** Every rung passed. Nothing measured a failure, so nothing says where one is. It is set by the four-bytes-per-thread cost of complete verification (a 1 GiB buffer) and by covering the widest tensor in the conformance corpus (Qwen3-0.6B's `151,936 x 1,024` embedding, about `2^27.2`), and it sits sixteen times below the MSL ceiling.

### The identity step, executed completely

The value **and** the source moved together: `4` → `268_435_456`, and `TargetFactSource::external_guarantee(…dispatch-threads)` → the profile's `TargetCompileProfileMeasurementSource`. Recomputed on this tree:

| Pin | Before | After |
| --- | --- | --- |
| Canonical descriptor length | 2,149 | **1,999** |
| Standard Metal artifact identity | `124981346c0bd593f19154f7ec3df26588179e0c7b446a995bbe4a7a92ba25bd` | **`3f98afa59d9ef46999acc211f2153a7d194444f5be3d0dd946f4128b57674a69`** |
| Standard Metal cache subject | `94dfde30611c9021da8e4a71f9b6824f3af1ff09ec68daa4c65d05bfc63e6370` | **`8bca5e7825cdd1dc37da5135b0ea7d6dbd3e9ce1557097f2ee9e60e79fe23d07`** |

That is the complete set. The profile **key is unchanged** — the same two dtypes over the same macOS Apple9 MSL 4.0 row — and the descriptor **shrank**, which is informative: the bound is a fixed-width `u64` so its value moves no bytes, and what moved is the source table, where retiring the SDK dispatch reference (the grid row's only user) removed a whole `external_guarantee` record while the measured source the row joined was already present.

No pinned identity outside `crates/tiler-build` moves. `crates/tiler-compiler`'s `the_governed_descriptor_bytes_do_not_move` and the `tiler-explain-v7 request=fb0b64dd69649785` digest both derive from `TargetProfileBuilder::governed`, which is deliberately unmoved. `tiler-macros`, `tiler`, `tiler-runtime`, `tiler-cache`, `tiler-artifact`, and the prototypes assert only computed-against-computed values.

### The trigger test named by this ticket could not have fired, and why

**Fact, and it corrects this ticket's own premise.** `target::tests::only_one_shape_admits_all_three_reduction_strategies` reads its bound from `TargetProfileBuilder::governed` — the *target-neutral prototype baseline* keyed `tiler.prototype-target-neutral-baseline.v1` — while calibration measures against `BoundMetalCompileDeclaration::first_macos_apple9`. Both declared four, so the difference was invisible. `tiler-compiler` cannot see `tiler-build`, so the test could not be repointed.

**The prototype row is deliberately not moved.** A macOS Apple9 device measurement is evidence about one target; a baseline standing in for every target cannot be widened by it, and widening it on the compiler's own say-so is exactly the unsourced number this ledger exists to refuse. The test is therefore renamed to what it actually checks, `the_prototype_baseline_admits_one_three_strategy_shape`, with its doc recording the distinction, and the real trigger is added where it can read the right profile: `tiler_build::metal_plan::tests::the_measured_grid_axis_admits_more_than_one_three_strategy_shape`. It observes the domain by compiling, mutation-proved in both directions — forcing the threshold reported `[(1, 4), (1, 8), (2, 4), (4, 16), (64, 64)]`, and its refusal half was proved bound-sensitive at the boundary (`268,435,456` work items compile; `536,870,912` refuse on `grid-axis`).

### Rerun of the retained sweep

Read-only, not retained here — recording a new result under `spikes/program-planning/` belongs to the calibration ticket. Rerun unchanged: **24 of 36 shapes retain all three strategies (was 1), with zero grid-axis refusals (was 23)**, and every one of the 23 previously grid-axis-refused shapes is now in the domain. The twelve that retain one alternative are the contributor counts admitting no balanced exact partition; they previously failed the whole batch and now return a portfolio, which is `correct-the-declined-strategy-record-for-an-unsplittable-reduction` (done, `f8c6c6e6`) rather than anything this ticket changed.

### Boundary

One profile, one environment. The row is `MeasuredEnvironment`-valid and is **not** a portable guarantee, an Apple-family claim, or a statement about any other OS row, GPU family, dtype, or toolchain. It establishes a floor and never a maximum. The evidence is exhaustive below 2,049 and sampled above, so the guarantee between two sampled rungs is an interpolation. Nothing was timed: no performance claim of any kind is made.

### Scopes added, and why

- `contracts/optimizer` — `docs/compiler/fusion-and-scheduling.md` asserted that the calibration is unobtainable, named this ticket as the unblocker, and named the trigger test that this ticket renames. Leaving it would leave a contract document asserting the thing this ticket disproved. Unheld by any live ticket at the time of the edit.
- `contracts/navigation` (shared) — `spikes/README.md`'s experiment catalog must gain the new spike in the same change that adds it. That scope is held by `re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal`; file-level disjointness was verified against that worker's actual branch with `git diff --name-only $(git merge-base tkt/re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal HEAD) tkt/re-own-or-close-the-open-questions-whose-owner-tickets-are-terminal`, which lists `docs/open-questions.md`, `docs/roadmap.md`, and three ticket files, and does not touch `spikes/README.md`.

Both are declaration and scheduling metadata for work this ticket already owned; neither expands the product outcome.
