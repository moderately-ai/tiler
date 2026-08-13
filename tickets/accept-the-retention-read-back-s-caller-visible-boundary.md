---
id: accept-the-retention-read-back-s-caller-visible-boundary
title: Accept the retention read-back's caller-visible boundary
status: in-progress
priority: p2
dependencies: [preserve-retained-tool-bytes-in-macro-read-back]
related: [emit-from-a-populated-retention-in-the-inline-expansion, measure-repeated-retention-note-cost-before-adding-deduplication]
scopes: [implementation/frontend]
shared_scopes: [project/tickets, contracts/navigation, contracts/integrations]
paths: []
tags: [decision, public-boundary]
claimed_from: todo
assignee: worker-retention-read-back
lease_expires_at: 1786665469
---
## Fact audit at `b35bb34fa4fc12131fcae1a2d973c060e56c2a57`

Re-read at this base: `RetainedText`, `SpokenRetention`, `report_retained_output`, `aot::deliver`, `docs/status.md`, `docs/integration/frontends.md`, and [`preserve-retained-tool-bytes-in-macro-read-back`](preserve-retained-tool-bytes-in-macro-read-back.md) (`done`). Each claim is judged against the file, not the last summary.

**Fact — no public item was added.** Verified. `SpokenRetention` and `report_retained_output` are `pub(crate)`. The note is caller-visible behaviour, not a new publicly reachable namespace.

**Fact — one ungated, nonfatal stderr note on every speaking resolution.** Verified. `aot::deliver` matches `Resolution::Hit`, `Published`, and `Uncached` with no wildcard and calls `report_retained_output` on the success path only. `spoken` selects on `RetainedText::is_empty`, run by run, never `DebugRetention::is_empty`. A closed or failing standard error is ignored.

**Fact — the read-back is byte-faithful.** Verified at this base. `SpokenRetention::write_to` writes `run.as_bytes()` after `{label}: ` and then the typed markers from `is_valid_utf8` / `is_truncated`. `RetainedText::Display` is unchanged and still uses `String::from_utf8_lossy(...).trim()`; it is the cache's public lossy view and is not this path. Tests pin leading whitespace, trailing whitespace, invalid bytes, multi-line text, truncation, elision, quiet, and no-run silence.

**False accepted-ticket premise — the landed read-back is not verbatim.** Historical of the pre-repair renderer; **false of this base.** The 2026-08-11 correction that named `RetainedText::Display` as the read-back is now a record of the defect [`preserve-retained-tool-bytes-in-macro-read-back`](preserve-retained-tool-bytes-in-macro-read-back.md) closed.

**False current message — it claims a later phase already succeeded.** Historical of the original preamble; **false of this base's `PREAMBLE`.** The note says "Offline compilation plus cache/artifact acceptance succeeded — later frontend emission can still refuse." Tests pin the retired phrases "The expansion succeeded" and "compiled, validated, and embedded" absent. **Still false of two comments at the start of this close-out:** the module header said "validated, embedded artifact", and `aot::deliver` said "the bytes being embedded". Those overclaims are repaired in this close-out.

**False documentation — the caller-visible boundary is still a labelled draft.** Verified as false of the 2026-08-11 decision and true of the docs at the start of this close-out. `docs/status.md` said the note "landed as a labelled draft awaiting Tom's acceptance" and was `awaiting-decision`. `docs/integration/frontends.md` said the note is "ungated as delivered" with the gate "Tom's open decision" and "a labelled draft, not an accepted boundary". Those sentences are repaired here.

**Missing at this base — a later-frontend-refusal subject perturbation.** The preamble pin existed; no test sequenced a written note with the later `DeliveryPlan::new` constructor `deliver` runs after `report_retained_output`. Added in this close-out.

## Accepted — 2026-08-11

**Decision.** Tom accepted the coordinator's ranked recommendation in the Codex coordination thread by replying `sounds good, accept`. The relay source is Tom's direct response in that thread. The behavior is accepted as an ungated, nonfatal, byte-faithful note.

1. **Every speaking run is reported on every resolution.** Hits, publications, and uncached resolutions all read the retention they actually resolved. No process-wide gate, cross-expansion deduplication, environment variable, `cfg`, attribute, or build-profile selector suppresses a non-empty run.
2. **Quiet means no retained bytes, not no runs.** A completely quiet Metal compilation retains named empty runs and writes nothing. An absent retention also writes nothing. The predicate remains per-run `RetainedText::is_empty`, never `DebugRetention::is_empty`.
3. **The note is nonfatal and best-effort.** Retention exists only after compilation succeeded and the artifact was validated. A closed or failing standard error does not invalidate that artifact, and a tool warning never becomes `compile_error!`.
4. **The tool bytes are byte-faithful.** The read-back writes every retained byte without trimming leading/trailing whitespace or substituting invalid UTF-8. Provenance, invalid-UTF-8 status, and truncation totals remain explicit metadata outside the tool's byte run. The repair uses `RetainedText::as_bytes` through a private frontend `io::Write` renderer. `RetainedText::Display` is unchanged.
5. **No span attribution.** The note remains attributed to `` `tiler::tensor!` `` on the expanding process's standard error, not to the region invocation span. Tiler's backend generated the MSL, so pointing at region syntax would direct the user to the wrong authority.

**Phase correction.** Reporting occurs after cache/artifact acceptance but before payload-cardinality checks, delivery-plan construction, token emission, and final token validation. The accepted behavior requires truthful attribution to the completed AOT/cache phase; it does not require moving reporting later. The preamble names that phase.

**Performance boundary.** Each retention is bounded to 16 runs of 16 KiB and the healthy Metal case scans two empty runs and performs no write. A speaking retention can be emitted repeatedly by rust-analyzer, so cumulative output is unbounded across expansions even though each expansion is bounded. No measurement currently justifies global state, hashing, or suppression. [`measure-repeated-retention-note-cost-before-adding-deduplication`](measure-repeated-retention-note-cost-before-adding-deduplication.md) is deferred until repeated output is measured; any future deduplication must be per-artifact/retention, bounded, and fail open by reporting rather than silently suppressing an unknown subject.

**Explicit exclusions.** No fatal warning, silent suppression, once-per-process flag, spanned diagnostic, machine-readable output, cache path, user selector, or unmeasured deduplication policy is accepted.

## Historical — what was awaiting acceptance

Landed as a **labelled draft** at merge `08714fd7` (worker commit `1b0d0614`), under AGENTS.md's rule that a tested public boundary stays a draft until Tom accepts its exact included and excluded surface. **No public item was added** — everything is `pub(crate)`, so under ADR 0075 this opens no new publicly reachable namespace. What returned it here was the *behaviour*, which a consumer sees.

### Included surface (accepted as stated, with the 2026-08-11 byte-faithful repair)

- One note, prefixed `` `tiler::tensor!`: ``, on the expanding process's standard error.
- Fires on **hits, publications and uncached resolutions alike** — a warm cache still shows what the tools said, rather than a diagnostic that existed once on whichever machine published first.
- **Never fatal.** A retention exists only where the compilation succeeded; a failing stage takes the existing family-scoped `compile_error!` path and never reaches this code.
- A quiet compilation and an empty retention both write **nothing**.
- The tool's own bytes reach the read-back beside `RetainedText`'s truncation and invalid-UTF-8 metadata. **Correction — 2026-08-11:** `verbatim` was false of the landed renderer. `RetainedText::Display` uses `String::from_utf8_lossy(...).trim()`, so it removes leading/trailing whitespace and substitutes invalid byte sequences before labelling them. [`preserve-retained-tool-bytes-in-macro-read-back`](preserve-retained-tool-bytes-in-macro-read-back.md) owned making the accepted byte-faithful behavior true and is `done`. **Correction — 2026-08-13:** that Display sentence is still true of the *cache* renderer and is no longer the read-back path. `SpokenRetention::write_to` writes `as_bytes()` and then the typed markers.

### Excluded surface

No environment variable, `cfg`, attribute or build profile gates it — the selection question stays closed, since the Metal producer retains unconditionally. No span-attributed diagnostic. No `compile_error!`. No once-per-process cap. No deduplication across expansions. No machine-readable form. No path printed to the cache entry.

### The one consequence that was weighed

**Under rust-analyzer, a region whose cached compilation carries a warning re-emits the note on every expansion request.** There is no gate, so this is zero in the healthy case and **unbounded in the defect case**. Tom accepted the ungated recommendation. The gate remains a three-line change to `reported_to` if practice later disagrees; that change is not this ticket.

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

## The question for Tom — answered 2026-08-11

**Accept the included and excluded surface above as stated?** The substantive sub-question was whether the note should carry a gate under rust-analyzer. **Accepted as built, ungated.** This ticket is implementation close-out of that decision, not a new packet.

## Perturbation evidence at this close-out

Assertions unchanged. Each perturbation was applied only to the named subject, then restored.

**Trim a leading byte** (`write_all(&run.as_bytes()[1..])`). `a_leading_whitespace_byte_is_written_exactly` failed:

```text
assertion `left == right` failed: the tool's own bytes must follow the run provenance unaltered
  left: [32, 112, 114, ...]
 right: [32, 32, 112, 114, ...]
```

Left is one space then `program_source`; right is the retained two-space run.

**Trim a trailing byte** (`write_all(&run.as_bytes()[..len - 1])`). `a_trailing_whitespace_byte_is_written_exactly` failed:

```text
assertion `left == right` failed: the tool's own bytes must follow the run provenance unaltered
  left: [..., 39, 120, 39, 32, 10]
 right: [..., 39, 120, 39, 32, 32]
```

Left ends `x` quote, one space, then the note's newline; right ends with the retained two trailing spaces.

**Replace an invalid byte** (`write_all(String::from_utf8_lossy(run.as_bytes()).as_bytes())`). `a_run_that_is_not_utf8_is_written_exactly_and_labelled` failed:

```text
assertion `left == right` failed: the tool's own bytes must follow the run provenance unaltered
  left: [239, 191, 189]
 right: [255, 254, 253]
```

Left is U+FFFD; right is the retained `0xff 0xfe 0xfd`. The rendered line was `tiler.metal.0.metal: ��� [output was not valid UTF-8]`.

**Drop the later frontend refusal** (`DeliveryPlan::new` no longer returns `ArtifactMissing` for a payload family with an empty envelope). `a_later_frontend_refusal_can_follow_a_written_retention_note` failed:

```text
a payload family with no envelope is the later check deliver runs after the note: DeliveryPlan { selection: ArtifactFamilySelection { policy: SelectedFamilies { families: [SelectedFamily { family: MacOs, ... }] } }, artifact: [], deliveries: [Payload] }
```

The note had already been written; the later constructor `deliver` runs after `report_retained_output` accepted the empty envelope instead of refusing it.

## Closes when

The byte-faithful read-back repair lands with deliberate subject perturbations for leading whitespace, trailing whitespace, invalid bytes, and a later frontend refusal after AOT acceptance; current quiet/multi-line/truncation behavior remains pinned; the preamble names only a phase that has actually completed; the source and status documentation call the caller-visible boundary accepted; and the exact-tip package and repository publication gates pass.

## Worker notes

Byte-faithful read-back and the narrowed preamble were already true at `b35bb34f` from [`preserve-retained-tool-bytes-in-macro-read-back`](preserve-retained-tool-bytes-in-macro-read-back.md). This close-out added the later-frontend-refusal subject perturbation, retired labelled-draft wording in source and status/contract docs, and repaired the module header that still said the artifact was embedded.

Checks at this worktree: `cargo test -p tiler-macros --lib retention` 15/15; `cargo test -p tiler-macros --lib` 185/186 with one ignored; `cargo test -p tiler-macros --doc` 0 doctests; `cargo clippy -p tiler-macros --all-targets -- -D warnings`; `RUSTDOCFLAGS="-D warnings" cargo doc -p tiler-macros --no-deps`; `tkt lint`; `git diff --check`; `make citations`; `make full` exit 0 (3554 nextest passed, 1236 release numerical tests passed). `tkt guard` against `origin/main` after commit. Not merged, not closed.
