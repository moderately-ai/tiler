---
id: emit-from-a-populated-retention-in-the-inline-expansion
title: Emit from a populated retention in the inline expansion
status: done
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

## Outcome — done, 2026-08-07

Landed at merge **`08714fd7`** (worker commit `1b0d0614`). `crates/tiler-macros/src/retention.rs` plus tests, wired into `aot::deliver`; 509 insertions across 4 files, all inside `implementation/frontend`.

A delivering expansion writes one note to standard error when the resolved entry's retained toolchain output carries bytes — on hits, publications and uncached resolutions alike, never fatal, silent for a quiet compilation. **The predicate is per-run, not `DebugRetention::is_empty`**, because the Metal producer records a silent stage as an *empty run*, so a quiet compilation is a retention of two runs for which `is_empty()` answers `false`; gating on it would print an empty header on every delivering expansion.

A spanned warning **was available and was declined**: `#![feature(proc_macro_diagnostic)]` with `Diagnostic::spanned(…, Level::Warning, …)` was tested on the pinned `nightly-2026-07-19` and works. It was rejected on attribution (no region text reaches the emitted MSL, so pointing at the invocation sends a consumer to edit something not at fault) and on testability (`Diagnostic::emit` writes where no test can read).

**Coordinator verification, independently reproduced:** no new public items; the `aot::deliver` call site matches all three `Resolution` variants with no wildcard; perturbing `spoken()` to `None` fails 9 of 12 tests and the naive `is_empty()` gate fails 4 of 12. One correction to the worker's report — those two perturbations cover **11 of 12**, not all twelve; `a_retention_with_no_runs_writes_nothing` survives both but is **not vacuous**, failing under a third targeted perturbation. `make full` exit 0 on the merged tree.

The caller-visible surface is a **labelled draft** and returns to Tom as [`accept-the-retention-read-back-s-caller-visible-boundary`](accept-the-retention-read-back-s-caller-visible-boundary.md), which carries the included/excluded surface and the one open sub-question (whether the note should be gated under rust-analyzer, where it re-emits per expansion request).
