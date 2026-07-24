---
id: share-the-spike-diagnostic-claims-verifier
title: Share one diagnostic-claims verifier across the spikes that retain claims
status: todo
priority: p3
dependencies: []
related: [record-gated-shape-spike-diagnostic-claims, verify-off-pin-shape-evidence-diagnostics, preserve-non-exhaustive-visibility-probe]
scopes: [research/shapes, research/extensions]
shared_scopes: [project/tickets]
paths: []
tags: [testing, harness, gate-reliability]
---
`record-gated-shape-spike-diagnostic-claims` asked its worker to prefer lifting `spikes/extensions/run.py`'s `verify_visibility_evidence` into a form both gated spikes share over copying it a third time. It copied instead, and stayed inside its declared `research/shapes` scope by doing so — the lift touches `research/extensions` and probably `implementation/workspace`, which that ticket did not hold. This ticket is the deferred half, filed so the choice is revisited rather than remembered.

**Measurement — how much is actually shared, at the merge of `tkt/record-gated-shape-spike-diagnostic-claims`.** `spikes/shapes/shape-evidence/verify_evidence.py` is 438 lines and `spikes/shapes/nightly-dependent-static-shapes/verify_claims.py` is 427. Eight functions appear in near-identical form in both: `read_text`, `read_pinned_channel`, `locked_package_version`, `sole_record`, `read_record`, `fixture_names`, `verify_failing_case`, and `verify_claims`, plus a per-file `*Failure` exception and a per-file `SCHEMA`/`MAX_RECORD_BYTES` pair.

**Fact — three differences are deliberate and must survive any lift, because each encodes the spike's gate posture.** They are documented in `verify_claims.py`'s module docstring and must not be flattened into one configurable predicate that a future edit can set wrongly:

- **The channel comparison is inverted.** The off-pin spike requires its recorded channel *not* to equal the repository pin, because its claim is about a compiler the gate cannot run. The gated spike requires equality, because the gate recompiles its fixtures on the pin and a record naming another compiler describes a run that no longer happens.
- **The gated spike digests nothing.** Compilation re-derives every `.stderr` on each gate invocation, so a digest pins nothing a rebuild would not already catch, and a legitimate pin migration regenerates them all. The off-pin spike digests both sources and diagnostics because no compilation ever re-derives them.
- **The gated spike checks `source_fragments` instead.** A source edit the compiler does not echo leaves the retained diagnostic byte-identical, so reproduction cannot see it; the fragment list states what a fixture must still contain to be the case it is recorded as.

**What closes this.** Either lift the eight shared functions into one module both spikes import — with the three differences above as explicit per-spike parameters whose *meaning* is documented at the shared site, not as bare booleans — and re-point both spikes and their tests at it; or record that the duplication is deliberate because the two postures diverge more than they share, and say so in both module docstrings so a third spike's author does not have to re-derive the answer. Do not lift by generalizing until the shared function no longer states either rule.

**Constraint.** Both spikes' test suites are adversarial — they mutate a copy of the spike and require the verifier to refuse. Whichever way this resolves, both suites must still fail for the same reasons afterwards: 31 cases under `spikes/shapes/nightly-dependent-static-shapes/` and the off-pin spike's own, all currently passing.

**Trigger for reconsideration if declined:** a third spike needs a retained-claims record, which is the point at which copying twice becomes copying three times.
