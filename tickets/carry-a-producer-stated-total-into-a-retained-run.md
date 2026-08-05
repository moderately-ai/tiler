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

**Fact — two equal bounds hide one fact between them.** `tiler_metal_aot::diagnostic::MAX_RETAINED_OUTPUT_BYTES` and `tiler_cache::expansion::MAX_RETAINED_RUN_BYTES` are both 16 KiB. `ToolOutput::capture` truncates at the first and records the tool's real total; `DebugRetention::retaining(label, bytes)` derives its total from the length it is handed. A stage that wrote 5 MB therefore reaches `retaining` as exactly 16384 bytes, and the stored run declares a total of 16384 — so `RetainedText::is_truncated` answers false for a run that is a prefix. Reproduce by reading `ToolOutput::capture` in `crates/tiler-metal-aot/src/diagnostic.rs` beside `DebugRetention::retaining` in `crates/tiler-cache/src/expansion/retention.rs`.

**Fact — the producer holds the missing number and cannot state it.** `retain-succeeding-metal-stage-tool-output` landed `stage_retention` in `crates/tiler-build/src/metal_cache.rs`, which has `ToolOutput::total_bytes()` in hand at the call and no parameter to put it in. Its doc names this gap and names this ticket.

**Inference — the fix belongs to `tiler-cache`, not to its callers.** A producer-side workaround is either a second bound (pre-truncating below the cache's, so the cache's own limit stops being the authority) or editing the tool's bytes to describe themselves (which is what byte-preserving capture exists to avoid). The decode path already validates `retained.len() <= total`, so the stored form needs no change: only the constructor does.

## Implementation keys

- A constructor taking the producer's own total beside the bytes, with `retaining` keeping its current meaning for a caller that has no separate total. Both must land in one bounded validation path rather than two.
- A total *below* the supplied length is a caller error, not a silent correction: `RetentionRejection::RetainedAboveTotal` already names that disagreement for stored bytes, and the refusal vocabulary should name it for a caller too.
- `stage_retention` states `ToolOutput::total_bytes()` once the constructor exists, and its "Where a truncation stops being visible" section is deleted rather than rewritten.

## Closes when

A run retained from an already-truncated `ToolOutput` reports the tool's real total on a validated cache hit, a test observes `is_truncated` answering true there, and the Metal producer's doc no longer describes the gap.
