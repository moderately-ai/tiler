---
id: drop-the-unread-sdk-path-from-the-resolved-toolchain
title: Drop the SDK path nothing reads from the resolved toolchain
status: done
priority: p3
dependencies: []
related: [avoid-toolchain-resolution-on-a-warm-expansion-cache-hit]
scopes: [implementation/metal-aot, implementation/build, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [research, performance]
---
## Why this exists

`Toolchain::resolve` spends one of its five `xcrun` invocations on `--show-sdk-path`, and the value it returns reaches no decision. Found while measuring the per-expansion toolchain-observation cost for `avoid-toolchain-resolution-on-a-warm-expansion-cache-hit`.

**Fact — the value is written and never read.** `crates/tiler-metal-aot/src/driver.rs:96` calls `self.sdk_field(sdk, "--show-sdk-path")` and stores it as `SdkIdentity::path` (`record.rs:35`), which travels into `ArtifactProvenance::sdk`. From there:

- compilation identity excludes it *by construction* — `push_sdk` destructures `path: _` (`identity.rs:396-406`), and `local_paths_are_excluded_from_the_subject` pins the exclusion;
- the artifact payload does not carry it — `PayloadSdkIdentity` holds `name`, `version`, and `build` only, and `crates/tiler-build/src/metal_assembly.rs:346-350` populates exactly those three;
- no compiler or linker flag reads it — `CompileRequest::compile_flags` emits `-target`, the triple, `-std=…`, the `-O` level, and the three numerical flags, and `link_flags` returns an empty vector (`input.rs:914-933`). The driver passes no `-isysroot`; `metal` selects its own SDK.

Reproduce the absence in one line: `grep -rn 'isysroot\|sdk\.path' crates/ --include='*.rs'` returns the construction site, the identity-exclusion test, and nothing that consumes the value.

**Measurement — macOS 27.0 arm64, Apple M4 Max, `nightly-2026-07-19`, 2026-08-01.** `xcrun --sdk macosx --show-sdk-path` costs ~6 ms warm, against a whole `Toolchain::resolve()` of 44–97 ms (median 52–63, n=40×3). So this is ~10% of the resolution and ~3% of a live in-region rust-analyzer edit's analysis round trip. Small, but it is the only part of the resolution that buys nothing at all: the other four `xcrun` answers and both `--version` executions each reach identity, provenance, or tool selection.

**Inference.** Either the field is dead and should go with its `xcrun` call, or it is provenance a reader is expected to have and the documentation should say who reads it. It cannot stay as an undocumented, unread public field whose population costs a subprocess on every expansion.

## Closes when

`SdkIdentity::path` and the `--show-sdk-path` invocation are removed together, leaving `Toolchain::resolve` at four `xcrun` calls; or the field is retained with a stated consumer and the documentation naming it.

**This is a public boundary and is Tom's.** `SdkIdentity` and `ArtifactProvenance` are public in `tiler-metal-aot`, so removing a public field is an ADR 0075 acceptance rather than a worker's cleanup — which is why the finding was filed instead of applied. Note that removal does *not* move any artifact identity: the field is already excluded from the subject, so no cache entry is invalidated and no golden moves. That is the property to re-verify before landing, not to assume.

## Outcome

**Tom accepted the removal on 2026-08-01 during the morning review**, choosing the first arm of "Closes when": `SdkIdentity::path` and the `--show-sdk-path` invocation are gone together, and `Toolchain::resolve` now makes four `xcrun` calls. That acceptance is the ADR 0075 public-boundary decision; nothing here self-accepted a public surface.

### The removal

The field's declaration (`record.rs`), the `sdk_field(sdk, "--show-sdk-path")` call and the `path:` initializer (`driver.rs`), the `path: _` arm of `push_sdk`'s destructure, and the identity fixture's `path` initializer (`identity.rs`) are the complete set: `cargo check --workspace --all-targets` compiles clean, so no other construction or read site existed. `PayloadSdkIdentity` in `tiler-artifact` was never involved — it holds `name`, `version`, and `build`, and `metal_assembly.rs` populates exactly those three, naming no path.

`SdkIdentity`'s doc now states why it carries no path — `metal` selects its own sysroot and the driver passes no `-isysroot` — so the next reader finds the reason where the field used to be rather than in this ticket.

### Six `xcrun` shim fixtures tightened, and why that needed two more scopes

Six test launchers spelled a `--show-sdk-path` arm. Five whitelist the driver's queries with `*) exit 1`; the sixth (`diverging_launcher`) falls through to the real `xcrun`. Removing the dead arm from all six is not tidying: it makes those fixtures refuse a reintroduced call. Verified rather than asserted — re-adding `self.sdk_field(sdk, "--show-sdk-path")?` to `resolve` makes `the_standard_metal_path_publishes_its_recorded_identities` fail with `SdkUnavailable { sdk: "macosx", ... xcrun exited exit status: 1 }`, and the change was reverted.

Four of the six live in `crates/tiler-build`, so `implementation/build` was added. `contracts/integrations` was added because `docs/integration/frontends.md` asserted in the present tense that a resolution makes five `xcrun` calls — a contract sentence this change falsifies. The dated measurement table there is left as measured and the paragraph below it now states the current count and what was removed. No scope was contended: the three other in-progress tickets hold `implementation/{ir,compiler,artifact}`, `research/{runtime,extensions}`, and `contracts/{decisions,navigation}`.

### No identity moved — proof, not inference

**Fact.** The compilation subject is byte-identical across the change. A temporary probe printed the fixture's `CompilationIdentity` bytes before and after: 405 bytes both times, and the same hex, opening `74696c65722e6d6574616c2d616f742e636f6d70696c6174696f6e2d6964656e746974792e7631` (`tiler.metal-aot.compilation-identity.v1`). The probe was removed before committing.

**Fact.** `the_standard_metal_path_publishes_its_recorded_identities` (`crates/tiler-build/src/metal_plan.rs`) is the decisive pin: it drives a real shimmed `Toolchain` through the standard path and asserts both recorded hex values — artifact identity `cee402b8…9f8853f` and cache subject `3f86db90…a65e58f3`. Both constants are untouched in the diff and both assertions pass, so neither the artifact identity nor the expansion-cache subject moved. `crates/tiler-metal/goldens` are likewise unmoved.

**Fact.** `cargo nextest run -p tiler-metal-aot -p tiler-build -p tiler-cache -p tiler-metal` reports 306 passed / 1 skipped both before the change (at `6f7caf3`) and after — the same counts, on the four packages that carry compilation identity, the expansion cache, the Metal goldens, and the payload assembly.

### `local_paths_are_excluded_from_the_subject` is retained, not retired

Its guarantee outlives the field. It relocated three paths; two remain, because `ResolvedTool::path` still reaches `CompilationIdentity::new` for both `metal` and `metallib`, so the test is not vacuous. Confirmed by watched failure: making `push_tool_version` fold `path` alongside `version` makes it fail with "relocating an identical toolchain must not change the key", and the perturbation was reverted. Its doc now records that the SDK path is excluded by `SdkIdentity`'s shape rather than by this test, and that `push_sdk`'s irrefutable destructure is what makes re-admitting one a compile error instead of a silent identity change.

### Measurement — macOS 27.0 arm64, Apple M4 Max, `rustc 1.99.0-nightly (eff8269f7 2026-07-18)`, 2026-08-01

Corroboration of the ticket's ~6 ms figure, by replaying the resolution's subprocess sequence — `--find metal`, `--find metallib`, both direct `--version` executions, then the SDK fields with and without `--show-sdk-path` — paired and interleaved to cancel thermal drift, 10 warm-up rounds then n=60 of each:

| sequence | min | median | mean | max |
| --- | ---: | ---: | ---: | ---: |
| five `xcrun` + two `--version` | 49.3 ms | 55.0 ms | 64.4 ms | 115.6 ms |
| four `xcrun` + two `--version` | 42.1 ms | 49.3 ms | 56.6 ms | 110.1 ms |

Paired delta: median **+7.4 ms**, mean +7.8 ms, positive in 45 of 60 pairs. The before-median of 55.0 ms sits inside the ticket's measured 44–97 ms band for a whole resolution, and the saving matches its ~6 ms estimate.

**Measurement boundary.** This times the subprocess sequence from a Python driver, not `Toolchain::resolve` in Rust, so it excludes the driver's own trimming and error handling and includes Python's spawn overhead; it corroborates the magnitude, and does not restate the ticket's in-process figure. A first, non-interleaved attempt is why the pairing matters: run as two sequential batches the medians were indistinguishable (81.8 vs 81.9 ms) with maxima near 179 ms, host variance entirely swamping a 7 ms effect. Nothing was measured about `cargo check` wall time here, so the ~10% and ~3% shares in "Why this exists" stand on the original measurement, not this one.

### Graph maintenance

`avoid-toolchain-resolution-on-a-warm-expansion-cache-hit` carries a dated note under "Filed, not absorbed" recording that this landed; its own measurements stay as measured at `a37be43`. `assemble-the-metal-payload-from-emission-and-compilation` carries a dated parenthetical where its outcome names `SdkIdentity::path` as a reason. `prototypes/serial-sum-compile`'s `the_payload_subject_carries_no_local_path` doc no longer cites the removed field and now says why the test survives it. No new ticket was filed: nothing out of scope was found.
