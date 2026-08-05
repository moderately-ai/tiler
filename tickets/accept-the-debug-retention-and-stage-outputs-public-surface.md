---
id: accept-the-debug-retention-and-stage-outputs-public-surface
title: Accept the debug-retention and stage-outputs public surface
status: done
priority: p2
dependencies: []
related: [retain-canonical-msl-under-a-debug-expansion-cache-entry, retain-succeeding-metal-stage-tool-output]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## The decision

Tom accepts or amends the public surface the two retention tickets landed as reviewed drafts on 2026-08-05:

From `retain-canonical-msl-under-a-debug-expansion-cache-entry` (commit `49a6fd7c`): `tiler-cache` — `DebugRetention`, `RetainedText`, `RetentionRefusal`, `RetentionRejection`, three `MAX_*` bounds, `BundleSection::DebugRetention`, `BundleRejection::RetainedDebug`, `ExpansionCache::get_or_publish_retaining`, `CachedEntry::retained_debug`, a `retained` field on `Resolution::Uncached`; `tiler-build` — `CompiledPayloads` and the neutral seam's compile closure returning one. The three identity answers (outside the key, inside the digest set, absent-is-a-hit) are implemented as mechanisms with present/absent/damaged each exercised.

From `retain-succeeding-metal-stage-tool-output` (commit `7bd91ec9`): `tiler-metal-aot` — `record::StageOutputs` (non-exhaustive, per-stage `ToolOutput`), `CompiledArtifact.stage_outputs`, `CompileStage::ALL`; `tiler-build` — `CompiledMetalPayload::stage_outputs()` returning `Some` only for an in-process compile. Attribution is by binding rather than run order, and identity is derived before either tool runs so tool text is unreachable from it by construction.

Known follow-up already filed: `carry-a-producer-stated-total-into-a-retained-run` (double-truncation reads as untruncated at exactly the shared bound).

Filed at `awaiting-decision`: only Tom closes an acceptance ticket.

## Delta appended before decision — the producer-stated total

From `carry-a-producer-stated-total-into-a-retained-run`, commit `c39cb814`, `tiler-cache`: `DebugRetention::retaining_with_stated_total(&str, &[u8], usize) -> Result<Self, RetentionRefusal>` (new public method; `usize` so `ToolOutput::total_bytes()` passes with no cast, stored field stays `u64` for framing width), and `RetentionRefusal::RetainedAboveTotal { retained: usize, total: usize }` (new variant on the already-non-exhaustive enum). Both share the single bounded validation path; the stored form is unchanged.

## Decided — accepted, delta included

Accepted by Tom on 2026-08-05 at the live decision review in the coordination session, witnessed first-hand by the coordinator, as "accept + delta": the landed surface plus the constructor above. No draft-labelling language exists in the retention code to release (`grep -rn "reviewed draft\|until Tom" crates/tiler-cache/src/expansion/retention.rs crates/tiler-metal-aot/src/record.rs` returns nothing), so the sweep is this record.
