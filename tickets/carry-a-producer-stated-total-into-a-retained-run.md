---
id: carry-a-producer-stated-total-into-a-retained-run
title: Carry a producer-stated total into a retained run
status: done
priority: p3
dependencies: []
related: [retain-succeeding-metal-stage-tool-output]
scopes: [implementation/cache, implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [implementation, diagnostics, cache]
---
## User-visible outcome

A retained run whose bytes were truncated by its producer reports the total the producer had, so a reader of a cache hit can tell a bounded prefix from a whole diagnostic.

## Why this exists

**Correction — 2026-08-10.** At base `c99ac54950f2` (and earlier landing `c39cb814` per sibling tickets' prose) this ticket is delivered: `DebugRetention::retaining_with_stated_total` exists in `crates/tiler-cache/src/expansion/retention.rs`, Metal `stage_retention` in `crates/tiler-build/src/metal_cache.rs` passes `ToolOutput::total_bytes()`, and the former gap section is gone — module docs now open with "**The stage's own total is stated, not re-derived.**" The Facts below are the filing-time problem statement; they are not live inventory of current source. Close condition is met; follow-on producer-side hit-path pairing was split to `make-stage-retention-reachable-from-a-test` (done).

**Fact — two equal bounds hide one fact between them (pre-fix failure mode of `retaining` alone).** `tiler_metal_aot::diagnostic::MAX_RETAINED_OUTPUT_BYTES` and `tiler_cache::expansion::MAX_RETAINED_RUN_BYTES` are both 16 KiB. `ToolOutput::capture` truncates at the first and records the tool's real total; `DebugRetention::retaining(label, bytes)` derives its total from the length it is handed. A stage that wrote 5 MB therefore reaches `retaining` as exactly 16384 bytes, and the stored run declares a total of 16384 — so `RetainedText::is_truncated` answers false for a run that is a prefix. That remains true of the length-derived path; the product path states the producer total instead. Reproduce by reading `ToolOutput::capture` in `crates/tiler-metal-aot/src/diagnostic.rs` beside `DebugRetention::retaining` / `retaining_with_stated_total` in `crates/tiler-cache/src/expansion/retention.rs`.

**~~Fact~~ — historical at filing: the producer held the missing number and could not state it.** `retain-succeeding-metal-stage-tool-output` landed `stage_retention` in `crates/tiler-build/src/metal_cache.rs`, which then had `ToolOutput::total_bytes()` in hand at the call and no parameter to put it in. Its doc then named this gap and named this ticket. **Correction — 2026-08-10.** That present-tense inventory is false at this base: `stage_retention` calls `retaining_with_stated_total(&stage_label(...), output.as_bytes(), output.total_bytes())`, and the Metal module doc no longer describes an open gap or names this ticket.

**Inference — the fix belongs to `tiler-cache`, not to its callers.** A producer-side workaround is either a second bound (pre-truncating below the cache's, so the cache's own limit stops being the authority) or editing the tool's bytes to describe themselves (which is what byte-preserving capture exists to avoid). The decode path already validates `retained.len() <= total`, so the stored form needs no change: only the constructor does.

## Implementation keys

- A constructor taking the producer's own total beside the bytes, with `retaining` keeping its current meaning for a caller that has no separate total. Both must land in one bounded validation path rather than two.
- A total *below* the supplied length is a caller error, not a silent correction: `RetentionRejection::RetainedAboveTotal` already names that disagreement for stored bytes, and the refusal vocabulary should name it for a caller too.
- `stage_retention` states `ToolOutput::total_bytes()` once the constructor exists, and its "Where a truncation stops being visible" section is deleted rather than rewritten.

## Closes when

A run retained from an already-truncated `ToolOutput` reports the tool's real total on a validated cache hit, a test observes `is_truncated` answering true there, and the Metal producer's doc no longer describes the gap.
