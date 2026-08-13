---
id: accept-the-retention-read-back-s-caller-visible-boundary
title: Accept the retention read-back's caller-visible boundary
status: in-progress
priority: p2
dependencies: [preserve-retained-tool-bytes-in-macro-read-back]
related: [emit-from-a-populated-retention-in-the-inline-expansion, measure-repeated-retention-note-cost-before-adding-deduplication]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
claimed_from: todo
assignee: worker-retention-read-back
lease_expires_at: 1786665469
---
## What is awaiting acceptance

Landed as a **labelled draft** at merge `08714fd7` (worker commit `1b0d0614`), under AGENTS.md's rule that a tested public boundary stays a draft until Tom accepts its exact included and excluded surface. **No public item was added** — everything is `pub(crate)`, so under ADR 0075 this opens no new publicly reachable namespace. What returns it here is the *behaviour*, which a consumer sees.

A delivering `tiler::tensor!` now writes one note to the expanding process's standard error whenever the resolved cache entry's retained toolchain output actually carries bytes.

### Included surface

- One note, prefixed `` `tiler::tensor!`: ``, on the expanding process's standard error.
- Fires on **hits, publications and uncached resolutions alike** — a warm cache still shows what the tools said, rather than a diagnostic that existed once on whichever machine published first.
- **Never fatal.** A retention exists only where the compilation succeeded; a failing stage takes the existing family-scoped `compile_error!` path and never reaches this code.
- A quiet compilation and an empty retention both write **nothing**.
- The tool's own bytes reach the read-back beside `RetainedText`'s truncation and invalid-UTF-8 metadata. **Correction — 2026-08-11:** `verbatim` was false of the landed renderer. `RetainedText::Display` uses `String::from_utf8_lossy(...).trim()`, so it removes leading/trailing whitespace and substitutes invalid byte sequences before labelling them. [`preserve-retained-tool-bytes-in-macro-read-back`](preserve-retained-tool-bytes-in-macro-read-back.md) owns making the accepted byte-faithful behavior true.

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

## Accepted — 2026-08-11

**Decision.** Tom accepted the coordinator's ranked recommendation in the Codex coordination thread by replying `sounds good, accept`. The relay source is Tom's direct response in that thread. The behavior is accepted as an ungated, nonfatal, byte-faithful note; this ticket moves to `todo` until the landed normalization is repaired and the labelled-draft wording is retired.

1. **Every speaking run is reported on every resolution.** Hits, publications, and uncached resolutions all read the retention they actually resolved. No process-wide gate, cross-expansion deduplication, environment variable, `cfg`, attribute, or build-profile selector suppresses a non-empty run.
2. **Quiet means no retained bytes, not no runs.** A completely quiet Metal compilation retains named empty runs and writes nothing. An absent retention also writes nothing. The predicate remains per-run `RetainedText::is_empty`, never `DebugRetention::is_empty`.
3. **The note is nonfatal and best-effort.** Retention exists only after compilation succeeded and the artifact was validated. A closed or failing standard error does not invalidate that artifact, and a tool warning never becomes `compile_error!`.
4. **The tool bytes are byte-faithful.** The read-back writes every retained byte without trimming leading/trailing whitespace or substituting invalid UTF-8. Provenance, invalid-UTF-8 status, and truncation totals remain explicit metadata outside the tool's byte run. The repair may use `RetainedText::as_bytes`; it does not need to change the already-accepted public `RetainedText::Display` contract unless a source audit proves that is the coherent single-authority change.
5. **No span attribution.** The note remains attributed to `` `tiler::tensor!` `` on the expanding process's standard error, not to the region invocation span. Tiler's backend generated the MSL, so pointing at region syntax would direct the user to the wrong authority.

**Phase correction.** The landed preamble's claim that “The expansion succeeded” and the artifact is “embedded” is false at its current call site: reporting occurs after cache/artifact acceptance but before payload-cardinality checks, delivery-plan construction, token emission, and final token validation. The accepted behavior requires truthful attribution to the completed AOT/cache phase; it does not require moving reporting later if narrowing the prose states that phase exactly.

**Performance boundary.** Each retention is bounded to 16 runs of 16 KiB and the healthy Metal case scans two empty runs and performs no write. A speaking retention can be emitted repeatedly by rust-analyzer, so cumulative output is unbounded across expansions even though each expansion is bounded. No measurement currently justifies global state, hashing, or suppression. [`measure-repeated-retention-note-cost-before-adding-deduplication`](measure-repeated-retention-note-cost-before-adding-deduplication.md) is deferred until repeated output is measured; any future deduplication must be per-artifact/retention, bounded, and fail open by reporting rather than silently suppressing an unknown subject.

**Explicit exclusions.** No fatal warning, silent suppression, once-per-process flag, spanned diagnostic, machine-readable output, cache path, user selector, or unmeasured deduplication policy is accepted.

## Closes when

The byte-faithful read-back repair lands with deliberate subject perturbations for leading whitespace, trailing whitespace, invalid bytes, and a later frontend refusal after AOT acceptance; current quiet/multi-line/truncation behavior remains pinned; the preamble names only a phase that has actually completed; the source and status documentation call the caller-visible boundary accepted; and the exact-tip package and repository publication gates pass.
