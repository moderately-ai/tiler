---
id: accept-the-retention-read-back-s-caller-visible-boundary
title: Accept the retention read-back's caller-visible boundary
status: awaiting-decision
priority: p2
dependencies: []
related: [emit-from-a-populated-retention-in-the-inline-expansion]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## What is awaiting acceptance

Landed as a **labelled draft** at merge `08714fd7` (worker commit `1b0d0614`), under AGENTS.md's rule that a tested public boundary stays a draft until Tom accepts its exact included and excluded surface. **No public item was added** — everything is `pub(crate)`, so under ADR 0075 this opens no new publicly reachable namespace. What returns it here is the *behaviour*, which a consumer sees.

A delivering `tiler::tensor!` now writes one note to the expanding process's standard error whenever the resolved cache entry's retained toolchain output actually carries bytes.

### Included surface

- One note, prefixed `` `tiler::tensor!`: ``, on the expanding process's standard error.
- Fires on **hits, publications and uncached resolutions alike** — a warm cache still shows what the tools said, rather than a diagnostic that existed once on whichever machine published first.
- **Never fatal.** A retention exists only where the compilation succeeded; a failing stage takes the existing family-scoped `compile_error!` path and never reaches this code.
- A quiet compilation and an empty retention both write **nothing**.
- The tool's own bytes are reproduced **verbatim**, multi-line diagnostics intact, carrying `RetainedText`'s truncation and invalid-UTF-8 markers.

### Excluded surface

No environment variable, `cfg`, attribute or build profile gates it — the selection question stays closed, since the Metal producer retains unconditionally. No span-attributed diagnostic. No `compile_error!`. No once-per-process cap. No deduplication across expansions. No machine-readable form. No path printed to the cache entry.

### The one consequence worth weighing before accepting

**Under rust-analyzer, a region whose cached compilation carries a warning re-emits the note on every expansion request.** There is no gate, so this is zero in the healthy case and **unbounded in the defect case**. The worker judged that better than suppressing distinct per-region diagnostics — its two siblings (`preflight`, `eviction`) do carry once-per-process flags, but their messages are process-scoped facts where a repeat is pure noise, whereas a retention is per-compilation and a gate would silence a second region on the strength of the first. If Tom disagrees, the gate is a three-line change to `reported_to`.

### Two design choices that were made against available alternatives

- **A spanned warning was available and was declined.** `#![feature(proc_macro_diagnostic)]` with `Diagnostic::spanned(…, Level::Warning, …).emit()` was **tested** on the pinned `nightly-2026-07-19` and does compile and render without failing the build. It was rejected on attribution — no region text reaches the emitted MSL, so pointing at `tiler::tensor! { … }` sends a consumer to edit a region that is not at fault — and on testability, since `Diagnostic::emit` writes where no test can read.
- **The predicate is per-run, not `DebugRetention::is_empty`.** The Metal producer names every stage and records a silent one as an **empty run**, so a completely quiet compilation is a retention of two runs for which `is_empty()` answers `false`. Gating on it would print a header with nothing under it on every delivering expansion.

## Coordinator verification, 2026-08-07 — independently reproduced, not relayed

- **No new public items.** `git diff` over `crates/tiler-macros/` shows zero added bare-`pub` items; two `pub(crate)`.
- **Call site is exhaustive.** `aot::deliver` matches all three `Resolution` variants with no wildcard, so a new variant is a compile error rather than a silent miss. It sits on the success path only.
- **The deliberate-failure claims reproduce exactly.** Perturbing `spoken()` to return `None` fails **9 of 12**; the naive `is_empty()` gate fails **4 of 12**, including `a_quiet_compilation_writes_nothing_though_its_retention_is_not_empty`.
- **One correction to the worker's report.** It claimed every test fails against at least one wrong implementation. The two perturbations cover **11 of 12** — `a_retention_with_no_runs_writes_nothing` survives both. It is **not vacuous**: a third targeted perturbation (unconditional `Some`) fails it. So the tests are sound and only the coverage summary was imprecise.
- **Gate green on the merged tree**, not on the branch: `make full` exit 0 at `08714fd7`, including 1,068 release numerical tests and workspace rustdoc with warnings denied.
- `tkt guard` exit 0, no under-declaration; the four warned collisions are declared-area only, and no other live branch touches `crates/tiler-macros/`.

## The question for Tom

**Accept the included and excluded surface above as stated?** The substantive sub-question, if the rest is uncontroversial: **should the note carry a gate under rust-analyzer**, trading unbounded repetition in the defect case against silencing distinct per-region diagnostics?

**Recommendation: accept as built, ungated.** A repeated note in the defect case is noise a developer can act on, whereas a gate silently drops a *different* region's diagnostic, which is the failure mode this repository consistently treats as worse — a check or report that cannot say what it has to say. The counterpoint is real: rust-analyzer re-expands aggressively, so "unbounded" is not theoretical, and a developer who cannot fix the warning has no way to quiet it. If that lands badly in practice the gate is cheap and reversible.

**Release trigger:** Tom answers. Nothing depends on it; the code is landed and the behaviour is live either way, so this decides whether it stays as built rather than blocking anything.
