---
id: drop-the-unread-sdk-path-from-the-resolved-toolchain
title: Drop the SDK path nothing reads from the resolved toolchain
status: in-progress
priority: p3
dependencies: []
related: [avoid-toolchain-resolution-on-a-warm-expansion-cache-hit]
scopes: [implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [research, performance]
claimed_from: todo
assignee: worker-sdk-drop
lease_expires_at: 1785594086
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
