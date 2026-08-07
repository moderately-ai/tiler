---
id: emit-from-a-populated-retention-in-the-inline-expansion
title: Emit from a populated retention in the inline expansion
status: in-progress
priority: p3
dependencies: [retain-succeeding-metal-stage-tool-output]
related: [retain-canonical-msl-under-a-debug-expansion-cache-entry]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, frontend, diagnostics]
claimed_from: todo
assignee: worker-retention
lease_expires_at: 1786137395
---
## Trigger check log

- 2026-08-05: not fired. The frontend has nothing to ask for yet — the Metal producer retains nothing, because `Toolchain::run_stage` drops a succeeding stage's output. Reproduce: `grep -n 'ToolOutput::capture' crates/tiler-metal-aot/src/driver.rs` shows the single capture site, inside the `!status.success()` arm.

## User-visible outcome

> **Retitled and re-scoped 2026-08-07, from "State a debug retention from the inline frontend".** The outcome below read: "An inline expansion built under a debug configuration can show the diagnostics its offline tools produced, read from the cache entry the compilation resolved to rather than from a recompile." The clause **"built under a debug configuration"** is the part that expired — it presumed a configuration selects retention, and the Metal backend has since made retention unconditional. The rest stands.

An inline expansion can show the diagnostics its offline tools produced, read from the cache entry the compilation resolved to rather than from a recompile. **No configuration gates it**, because the producer retains on every publication; the only open question is what, if anything, the expansion emits from what it finds.

## Why this exists

**Fact — the storage and the seam exist.** `retain-canonical-msl-under-a-debug-expansion-cache-entry` landed `tiler_cache::expansion::DebugRetention`, the `BundleSection::DebugRetention` frame, and `tiler_build::CompiledPayloads::retained`.

> **The clause that followed is struck. Corrected 2026-08-07.** It read: "Every producer in the workspace states `DebugRetention::none()`." **`crates/tiler-build/src/metal_cache.rs:403` now states a real retention** — `retained: stage_retention(&outputs)`, labelling each run `{BACKEND}.{delivery}.{stage.tool()}` with `retaining_with_stated_total`. Only the generic default and the `custom_backend` test producer still state none. So a frontend asking for retention today receives a **populated** section from the Metal producer, which is what fired this ticket's trigger.

**Fact — a debug configuration is a caller-stated input, and the caller is the frontend.** `tiler_cache` reads no environment and `tiler_build` reads none either, under the ADR 0089 root policy: names, parsing, and defaults stay with the frontend.

> **The question this ticket opened with has been answered the other way, and the scope narrows accordingly. Corrected 2026-08-07.** The struck clause read: "whatever selects retention — an environment variable, an attribute on the invocation, a `cfg` — is `crates/tiler-macros`' decision to make and its shape is the first question this ticket answers." **The Metal backend made retention unconditional and caller-independent**, documented at `crates/tiler-build/src/metal_cache.rs:435-440`: "**Always stated, never discovered.** This backend retains its stage output on every publication rather than consulting an environment variable or a build profile, which is the ADR 0089 root policy the retention module restates."
>
> That does not contradict the Fact above — the root policy still keeps names and defaults out of the lower crates — but it means **no selector is owed**, because nothing is selected. The two questions this ticket bundled are separable, the first is closed, and what remains is the second.

## What actually remains, after the 2026-08-07 correction

**Read-back, not selection.** A retention now arrives populated; nothing in the frontend does anything with it. `crates/tiler-macros` holds **no `DebugRetention` reference at all` — its only "retained" vocabulary is the failure-path `compile_error!` diagnostic, which is a different mechanism reached on a different path.

So the live question is: **what, if anything, does an inline expansion emit from a populated retention?** A note the caller sees at expansion time; a `compile_error!` under some opt-in; a path printed for reading by hand; or deliberately nothing, with the retention left to be read out of the cache directly. Each is a caller-visible frontend behaviour, so whichever is chosen is a public-boundary question under ADR 0075 and comes back to Tom with the built shape.

**Do not re-open the selection question.** If the answer turns out to need one after all — a per-invocation opt-in, say — that is a new decision against a backend that currently retains unconditionally, and it belongs in its own ticket rather than reinstated here.

**Inference — reading it back is a second, separable question.** Retention puts text into an entry; showing it to a consumer is an emitted diagnostic, and a `compile_error!` is fatal while a note is not. Whether an expansion emits anything at all from a retention, or whether the text is for a developer reading the cache by hand, is worth deciding before any of it is built.

## Trigger

`retain-succeeding-metal-stage-tool-output` reaching `done`, which is what first gives the Metal producer something to state. Filed `deferred` rather than `todo` because a frontend that asked for retention today would receive an empty section from every producer, and the board must not offer non-work.
- 2026-08-07 — **FIRED, and the ticket needs re-scoping before dispatch rather than only re-statusing.** Verified independently by the coordinator, not relayed: `retain-succeeding-metal-stage-tool-output` reads `status: done`, and the work landed rather than the status merely flipping — `grep -n 'ToolOutput::capture' crates/tiler-metal-aot/src/driver.rs` now returns **two** sites, `:304` inside the failure arm and **`:307` on the success path**, which is exactly the inversion the 2026-08-05 entry recorded as unfired. The producer is wired through: `crates/tiler-build/src/metal_cache.rs:403` states `retained: stage_retention(&outputs)`. So a frontend asking for retention today receives a populated section from the Metal producer, and the stated ground for deferral is gone.

  **Two stated Facts are now false and must be repaired before this is briefed.** (1) "Every producer in the workspace states `DebugRetention::none()`" — `metal_cache.rs:403` states a real retention; only the generic default and the `custom_backend` test producer still state none. (2) More consequentially, this ticket opens by naming the *selection* question — "whatever selects retention … is `crates/tiler-macros`' decision to make and its shape is the first question this ticket answers" — and the Metal backend has already answered it the other way: retention is **unconditional and caller-independent**, documented at `metal_cache.rs:435-440` as "**Always stated, never discovered.**" So the live remainder is this ticket's *second*, separable question — whether an inline expansion emits anything from a retention, and as what. `crates/tiler-macros` holds no `DebugRetention` reference at all; its only "retained" vocabulary is the failure-path `compile_error!` diagnostic, which is a different mechanism. **Recommend re-scoping to the read-back question before dispatch.** Recheck: `grep -n 'ToolOutput::capture' crates/tiler-metal-aot/src/driver.rs && grep -n 'retained: stage_retention' crates/tiler-build/src/metal_cache.rs`.
