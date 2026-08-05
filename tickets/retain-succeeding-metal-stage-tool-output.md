---
id: retain-succeeding-metal-stage-tool-output
title: Retain a succeeding Metal stage's tool output
status: in-progress
priority: p3
dependencies: []
related: [retain-canonical-msl-under-a-debug-expansion-cache-entry]
scopes: [implementation/metal-aot, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, diagnostics, cache]
claimed_from: todo
assignee: agent-metal-retain
lease_expires_at: 1785938306
---
## User-visible outcome

A Metal compilation that succeeds *and* warns can have the compiler's own words retained beside the artifact it produced, instead of the warning being discarded the moment the tool exits zero.

## Why this exists

**Fact — the driver keeps a stage's output only when the stage fails.** `Toolchain::run_stage` in `crates/tiler-metal-aot/src/driver.rs` runs the tool with `Command::output`, and on `!status.success()` returns `DriverError::ToolFailure` carrying `ToolOutput::capture(&output.stderr)`. On success it returns `Ok(())`, and both `output.stdout` and `output.stderr` are dropped there. Reproduce by reading that function; there is no other capture site, because `compile` calls `run_stage` twice and reads only its `Result`.

**Fact — the storage on the other side already exists.** `retain-canonical-msl-under-a-debug-expansion-cache-entry` landed `BundleSection::DebugRetention` and `tiler_cache::expansion::DebugRetention`, and threaded it through `tiler_build::CompiledPayloads::retained`, so a backend that *has* diagnostics can state them and read them back from a validated hit. `accept_or_publish_delivered_metal_artifact` states `DebugRetention::none()` today and names this ticket where it does.

**Inference — this is a `tiler-metal-aot` change, and it touches an accepted public boundary.** A succeeding compilation would have to carry its captured output out of `run_stage`, which means `CompiledArtifact` (or a value beside it) grows a field, and `ToolOutput` becomes reachable from a success path rather than only from `DriverError`. That is Tom's boundary to accept.

## Implementation keys

- Retain bytes, bounded exactly as `ToolOutput` already bounds them, with truncation recorded. Do not introduce a second bound or a second capture idiom.
- Both stages produce output. A retention that named one run for two stages would attribute the linker's words to the compiler; label them separately.
- Empty output is not the same as no output. A stage that wrote nothing is a fact worth carrying, and `RetainedText::is_empty` already distinguishes it.
- The captured text must not reach `PayloadMetadata`, any identity, or the cache subject: a warning is not a compilation input, and folding one would give two hosts two identities for one compilation.

## Closes when

A succeeding `metal` and `metallib` run's captured output is reachable from `tiler-build`, `accept_or_publish_delivered_metal_artifact` states it as a `DebugRetention`, and a test observes the retained text coming back from a validated cache hit rather than from the run that produced it.
