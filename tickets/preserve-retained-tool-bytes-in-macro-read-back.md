---
id: preserve-retained-tool-bytes-in-macro-read-back
title: Preserve retained tool bytes in macro read-back
status: in-progress
priority: p1
dependencies: [emit-from-a-populated-retention-in-the-inline-expansion]
related: [accept-the-retention-read-back-s-caller-visible-boundary]
scopes: [implementation/frontend]
shared_scopes: [project/tickets]
paths: []
tags: [frontend, diagnostics, correctness, public-boundary]
claimed_from: todo
assignee: worker-retained-tool-bytes
lease_expires_at: 1786585710
---
## User-visible outcome

The inline macro writes each retained tool byte without trimming or lossy substitution, while keeping provenance, invalid-UTF-8 status, and truncation metadata distinguishable from tool output.

## Facts to re-verify

**Fact — storage is already exact.** `RetainedText` stores `Vec<u8>` and exposes `as_bytes`; the cache frames and digests those bytes without interpreting them.

**False accepted-ticket premise — the landed read-back is not verbatim.** `SpokenRetention::fmt` delegates each run to `RetainedText::Display`, which renders `String::from_utf8_lossy(&self.retained).trim()`. Leading/trailing whitespace is removed and invalid byte sequences are substituted. The invalid-UTF-8 and truncation markers remain truthful, but the tool byte run is not exact.

**False current message — it claims a later phase already succeeded.** `aot::deliver` calls `report_retained_output` immediately after cache/artifact acceptance but before payload-cardinality validation, route-fact construction, `DeliveryPlan::new`, token emission, and `guarded_emission`'s final token validation. The note currently says “The expansion succeeded” and the artifact is “embedded”; a later typed refusal can still prevent both. The retained output is real, but its phase attribution overclaims.

## Required outcome

- Write the exact retained bytes through the existing `io::Write` seam, after the macro/run provenance and before separately distinguishable metadata.
- Preserve leading and trailing whitespace, embedded newlines, and invalid byte sequences exactly.
- Keep invalid-UTF-8 and truncation state explicit without inserting marker bytes into what is claimed as the tool's own run.
- Preserve silence for absent and all-empty retentions, all speaking runs in producer order, every resolution path, nonfatal behavior, and the `` `tiler::tensor!` `` attribution.
- Prefer a private frontend renderer over changing the accepted public `RetainedText::Display` surface unless the complete consumer census proves one shared renderer is the only coherent authority.
- Make the phase claim exact. Either carry the report to the final successful emission boundary, or keep it at AOT resolution and say only that offline compilation plus cache/artifact acceptance succeeded and that later frontend emission can still refuse. Prefer the latter narrow wording unless a complete control-flow audit finds callers rely on final-success timing.

## Subject perturbations

With assertions unchanged, independently trim a leading byte, trim a trailing byte, and replace an invalid byte; quote the exact failing diagnostics. Retain the existing multi-line, quiet, no-run, truncation, elision, and all-speaking-run census.

## Stop conditions

Stop if exact stderr bytes cannot coexist with an unambiguous metadata boundary, if the coherent repair changes the accepted public cache surface, or if truthful phase attribution requires a new caller-visible result shape. File the required public-boundary decision rather than silently redefining `RetainedText::Display` or expansion success.

## Required checks

Run the macro package tests and doctests, Clippy/rustdoc with warnings denied, exact consumer fixtures, citations, lint, exact-base guard, and the full exact-tip repository gate required by the touched crate path.
