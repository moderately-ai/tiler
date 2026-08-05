---
id: state-a-debug-retention-from-the-inline-frontend
title: State a debug retention from the inline frontend
status: deferred
priority: p3
dependencies: [retain-succeeding-metal-stage-tool-output]
related: [retain-canonical-msl-under-a-debug-expansion-cache-entry]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, frontend, diagnostics]
---
## Trigger check log

- 2026-08-05: not fired. The frontend has nothing to ask for yet — the Metal producer retains nothing, because `Toolchain::run_stage` drops a succeeding stage's output. Reproduce: `grep -n 'ToolOutput::capture' crates/tiler-metal-aot/src/driver.rs` shows the single capture site, inside the `!status.success()` arm.

## User-visible outcome

An inline expansion built under a debug configuration can show the diagnostics its offline tools produced, read from the cache entry the compilation resolved to rather than from a recompile.

## Why this exists

**Fact — the storage and the seam exist, and nothing states a retention.** `retain-canonical-msl-under-a-debug-expansion-cache-entry` landed `tiler_cache::expansion::DebugRetention`, the `BundleSection::DebugRetention` frame, and `tiler_build::CompiledPayloads::retained`. Every producer in the workspace states `DebugRetention::none()`.

**Fact — a debug configuration is a caller-stated input, and the caller is the frontend.** `tiler_cache` reads no environment and `tiler_build` reads none either, under the ADR 0089 root policy: names, parsing, and defaults stay with the frontend. So whatever selects retention — an environment variable, an attribute on the invocation, a `cfg` — is `crates/tiler-macros`' decision to make and its shape is the first question this ticket answers.

**Inference — reading it back is a second, separable question.** Retention puts text into an entry; showing it to a consumer is an emitted diagnostic, and a `compile_error!` is fatal while a note is not. Whether an expansion emits anything at all from a retention, or whether the text is for a developer reading the cache by hand, is worth deciding before any of it is built.

## Trigger

`retain-succeeding-metal-stage-tool-output` reaching `done`, which is what first gives the Metal producer something to state. Filed `deferred` rather than `todo` because a frontend that asked for retention today would receive an empty section from every producer, and the board must not offer non-work.
