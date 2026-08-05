---
id: accept-the-debug-retention-and-stage-outputs-public-surface
title: Accept the debug-retention and stage-outputs public surface
status: awaiting-decision
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
