---
id: record-gated-shape-spike-diagnostic-claims
title: Record the gated shape spike's diagnostic claims
status: done
priority: p2
dependencies: []
related: [verify-off-pin-shape-evidence-diagnostics, compile-extension-spike-fixtures-in-the-gate]
scopes: [research/shapes]
shared_scopes: [project/tickets]
paths: []
tags: [testing, gate-reliability, harness]
---
`verify-off-pin-shape-evidence-diagnostics` gave `spikes/shapes/shape-evidence` a diagnostics record because the Rust gate cannot compile it, and decided in the same pass that `spikes/shapes/nightly-dependent-static-shapes` does not need the same treatment. That decision closed one half of the question and left this half explicit.

**Fact — verified by reading, at `412ceae`.** `scripts/check_rust.py` names the nightly workspace in `GATED_SPIKE_WORKSPACES`, so the gate compiles it on the pinned nightly, and `verify_fixture_coverage` requires the run transcript to name every `tests/ui/*/*.rs` case found on disk. `conformance/tests/ui.rs` now also names the exact fixture inventory in both directions, so a case deleted from the tree fails rather than silently resolving the glob to fewer fixtures.

**The gap that leaves.** Compilation proves that a fixture and the `.stderr` beside it agree. It does not prove that the agreed diagnostic is still the claim ADR 0067 relies on. A fixture weakened until it fails for an unrelated reason, refreshed with `TRYBUILD=overwrite` in the same commit, compiles and passes: fixture and diagnostic agree, and nothing states which error the case is supposed to demonstrate. `spikes/extensions/non-exhaustive-visibility` carries both halves for exactly this reason — its record pins each case's first line, diagnostic code, required fragments, and forbidden fragments, and `spikes/extensions/run.py --self-test` checks them without invoking Cargo.

**Scope of the work.** Give the four nightly compile-fail cases the same semantic record. The gated form differs from the off-pin one already built under `spikes/shapes/shape-evidence`: its recorded channel must *equal* the repository pin rather than differ from it, and it must not digest the retained `.stderr` bytes, because a legitimate pin migration regenerates them and reproduction is the total check. That is exactly `verify_visibility_evidence`'s shape in `spikes/extensions/run.py`.

Prefer lifting that verifier into a form both gated spikes share over copying it a third time. Doing so touches `research/extensions` and probably `implementation/workspace`, which is why it is a separate ticket rather than part of the off-pin work: `verify-off-pin-shape-evidence-diagnostics` held `research/shapes` only.

**Trigger for reconsideration if declined again.** A pin migration that has to re-record a nightly `.stderr` by hand is the moment the missing claim bites, because nothing then distinguishes a diagnostic that moved from a claim that changed.

## Outcome

The four nightly compile-fail cases and the two compile-pass cases now carry a semantic record, in the gated form the ticket specified rather than a copy of the off-pin one.

**What landed.** `spikes/shapes/nightly-dependent-static-shapes/results/2026-07-24-nightly-2026-07-19.json` states, per case, the ADR clause it demonstrates, the diagnostic's exact first line, the whole ordered sequence of error codes the file emits, required and forbidden message fragments, and `source_fragments` the fixture must still contain. `verify_claims.py` checks it and `test_nightly_dependent_claims.py` checks the checker adversarially — 28 tampering cases, each mutating a copy of the spike and requiring a refusal, including the exact attack the ticket named: a fixture weakened until it fails for an unrelated reason and refreshed with `TRYBUILD=overwrite` in the same commit.

**The two gated-versus-off-pin differences the ticket called for are implemented and documented as reasoning, not as configuration.** The channel comparison is *inverted* — the recorded channel must **equal** `rust-toolchain.toml`'s pin, because the gate recompiles these fixtures with exactly that toolchain, so a record naming another compiler describes a run that no longer happens and a pin migration must force the claims to be re-derived rather than inherited. And nothing is digested, because compilation re-derives every `.stderr` on every gate invocation, so a digest pins nothing a rebuild would not already catch while a legitimate pin migration regenerates them all.

**A third rule was needed that neither the ticket nor the off-pin form has.** Reproduction cannot see a source edit the compiler does not echo: changing a type alias no diagnostic quotes, or deleting the higher ranks a compile-pass case exists to cover, leaves the whole gated suite green — both were measured on the governed pin. `source_fragments` is what covers that gap, and it is deliberately not a digest: it survives cosmetic edits a digest would reject and refuses semantic ones a digest could only report as "something changed".

Beyond the recorded claims, the verifier also fails closed when the record's `trybuild` version disagrees with the spike's `Cargo.lock` (trybuild normalizes the retained diagnostics, so another version is another measurement), when `measure.py`'s parsed `TOOLCHAINS` tuple or the README's documented nightlies omit the pin, when more than one record is retained, when a claim resolves to no accepted decision, when a fixture and the record disagree in *either* direction, and on an orphaned `.stderr`.

**Verification run in the worktree:** `verify_claims.py` exits 0 and resolves all six cases against ADR-0067 on `nightly-2026-07-19` (`rustc 1.99.0-nightly (eff8269f7 2026-07-18)`); `pytest spikes/shapes/nightly-dependent-static-shapes/` reports 31 passed; `ruff check` and `ruff format --check` pass, after reformatting the two new files — the interrupted worker had not run Ruff, and both needed it.

**Declined and ticketed, not silently skipped.** The ticket preferred lifting the verifier into a shared form over copying it a third time. It was copied: the lift touches `research/extensions` and probably `implementation/workspace`, neither of which this ticket declared. `share-the-spike-diagnostic-claims-verifier` owns the decision and carries the measurement — eight functions in near-identical form across two ~430-line files — together with the three differences that must survive any lift.
