---
id: record-gated-shape-spike-diagnostic-claims
title: Record the gated shape spike's diagnostic claims
status: in-progress
priority: p2
dependencies: []
related: [verify-off-pin-shape-evidence-diagnostics, compile-extension-spike-fixtures-in-the-gate]
scopes: [research/shapes]
shared_scopes: []
paths: []
tags: [testing, gate-reliability, harness]
claimed_from: todo
assignee: agent-record-gated-shape-spike-diagnostic-claims
lease_expires_at: 1784932575
---
`verify-off-pin-shape-evidence-diagnostics` gave `spikes/shapes/shape-evidence` a diagnostics record because the Rust gate cannot compile it, and decided in the same pass that `spikes/shapes/nightly-dependent-static-shapes` does not need the same treatment. That decision closed one half of the question and left this half explicit.

**Fact — verified by reading, at `412ceae`.** `scripts/check_rust.py` names the nightly workspace in `GATED_SPIKE_WORKSPACES`, so the gate compiles it on the pinned nightly, and `verify_fixture_coverage` requires the run transcript to name every `tests/ui/*/*.rs` case found on disk. `conformance/tests/ui.rs` now also names the exact fixture inventory in both directions, so a case deleted from the tree fails rather than silently resolving the glob to fewer fixtures.

**The gap that leaves.** Compilation proves that a fixture and the `.stderr` beside it agree. It does not prove that the agreed diagnostic is still the claim ADR 0067 relies on. A fixture weakened until it fails for an unrelated reason, refreshed with `TRYBUILD=overwrite` in the same commit, compiles and passes: fixture and diagnostic agree, and nothing states which error the case is supposed to demonstrate. `spikes/extensions/non-exhaustive-visibility` carries both halves for exactly this reason — its record pins each case's first line, diagnostic code, required fragments, and forbidden fragments, and `spikes/extensions/run.py --self-test` checks them without invoking Cargo.

**Scope of the work.** Give the four nightly compile-fail cases the same semantic record. The gated form differs from the off-pin one already built under `spikes/shapes/shape-evidence`: its recorded channel must *equal* the repository pin rather than differ from it, and it must not digest the retained `.stderr` bytes, because a legitimate pin migration regenerates them and reproduction is the total check. That is exactly `verify_visibility_evidence`'s shape in `spikes/extensions/run.py`.

Prefer lifting that verifier into a form both gated spikes share over copying it a third time. Doing so touches `research/extensions` and probably `implementation/workspace`, which is why it is a separate ticket rather than part of the off-pin work: `verify-off-pin-shape-evidence-diagnostics` held `research/shapes` only.

**Trigger for reconsideration if declined again.** A pin migration that has to re-record a nightly `.stderr` by hand is the moment the missing claim bites, because nothing then distinguishes a diagnostic that moved from a claim that changed.
