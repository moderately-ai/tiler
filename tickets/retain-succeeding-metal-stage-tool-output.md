---
id: retain-succeeding-metal-stage-tool-output
title: Retain a succeeding Metal stage's tool output
status: review
priority: p3
dependencies: []
related: [retain-canonical-msl-under-a-debug-expansion-cache-entry, carry-a-producer-stated-total-into-a-retained-run]
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

## Outcome

**Held at `review` for one public boundary, which is Tom's.** The work is complete and gated; what is not settled is the surface it grows, which this ticket predicted would be Tom's to accept.

**Surface delta.** `tiler_metal_aot::record::StageOutputs` is new — a `#[non_exhaustive]` record with public `metal` and `metallib` fields of the existing `ToolOutput`, plus `stage(CompileStage) -> &ToolOutput`. `CompiledArtifact` grows a `stage_outputs: StageOutputs` field, additive out of crate because the struct is already `#[non_exhaustive]`. `CompileStage::ALL` is new. `Toolchain::run_stage` (private) returns `ToolOutput` instead of `()`. In `tiler-build`, `CompiledMetalPayload` grows a private `Option<StageOutputs>` read through a new `stage_outputs()` accessor, `Some` for a payload compiled in this process and `None` for one rebuilt from a resolved object; the `pub(crate) into_content` it superseded is removed. No signature of any public `tiler-build` function changed.

**One run per stage per delivery position, always stated.** Labels are `tiler.metal.<delivery>.metal` and `tiler.metal.<delivery>.metallib`, so the linker's words cannot arrive under the compiler's name and a two-family selection is four runs rather than two fighting over two labels. A silent stage is retained as an empty run, because both stages ran. A retention the cache refuses — a selection past the 16-run limit is the reachable case — becomes one `tiler.metal.retention-elided` run rather than a failed build: a warning that could fail a correct compilation would be a compilation input.

**Evidence, and what it does not cover.** `a_real_front_end_warning_survives_a_succeeding_compilation` runs the *real* Apple toolchain: an unused local produces `warning: unused variable 'unused_local' [-Wunused-variable]`, retained through the success path, while the trivial control kernel compiles silently — so the driver-level capture is measured against the tool that motivated it. The cache round trip is exercised with fake tools only (`warning_toolchain` in `metal_plan.rs`), because the emitter's generated MSL does not warn and forcing the real compiler to warn through `accept_or_publish_metal_plan` would mean emitting deliberately bad MSL. **Unexercised:** a real `metal` warning travelling through publication to a validated hit, and any real `metallib` output at all — the linker wrote nothing in every observed run.

**Measurement boundary.** macOS 27.0.0, Xcode at `/Applications/Xcode.app/Contents/Developer`, `metal`/`metallib` as `xcrun --sdk macosx` resolves them on 2026-08-05. The observed warning text embeds the driver's per-process scratch path, so retained text is host- and run-specific; it reaches no key, which is why that is a diagnostic fact rather than a reuse defect.

**One gap filed rather than absorbed.** `carry-a-producer-stated-total-into-a-retained-run`: `ToolOutput` and `MAX_RETAINED_RUN_BYTES` bound a run identically at 16 KiB, so an already-truncated capture reaches `DebugRetention::retaining` at exactly the bound and the stored run declares that prefix as its whole. Closing it needs a `tiler-cache` constructor taking the producer's total, which is outside this ticket's scopes.
