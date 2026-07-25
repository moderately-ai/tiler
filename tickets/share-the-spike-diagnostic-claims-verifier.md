---
id: share-the-spike-diagnostic-claims-verifier
title: Share one diagnostic-claims verifier across the spikes that retain claims
status: done
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

## Outcome

**Declined, on a measurement that corrects this ticket's own.** The duplication is recorded as deliberate in both module docstrings, per the second option above. The divergence the duplication had already produced was closed rather than left.

**Retraction — "eight functions appear in near-identical form" is wrong for two of the eight, and they are the two that matter.** Measured mechanically at `9e51c6a` by extracting every top-level function from both files with `ast` and diffing same-named pairs after normalizing the exception class (`ClaimFailure`/`EvidenceFailure`). Eleven names are shared, not eight. Four are byte-identical: `read_text` (6 lines), `read_pinned_channel` (11), `locked_package_version` (15), `fixture_names` (3). Three differ only in a message string or which entrypoint they call: `read_record`, `sole_record`, `main`. The remaining four are not near-identical at all — `verify_toolchain` differs in 40 of its 40-46 lines, `verify_failing_case` in 32 of 46, `verify_claims` in 23 of 44, and `verify_compiling_case` takes a different signature. This ticket named `verify_failing_case` and `verify_claims` as shared; they are the two largest of the eight and the two that carry the postures. Reproduce with the extraction script's method against the two files at this commit.

**Why the lift was declined, on that measurement.** The three deliberate differences this ticket lists are real but are not the whole divergence. Unifying the four posture-bearing functions needs roughly seven parameters — channel-comparison direction, digesting, `source_fragments`, ADR attribution, claim-id uniqueness, singular-versus-plural case form, and empty-list strictness — each settable backwards by a later edit, replacing two files that each state their own rule in prose. That is the outcome this ticket's own constraint forbids.

Lifting only the seven posture-free helpers was considered separately and rejected on its own terms rather than by association. Every one would have to take the caller's exception type as a parameter, because both adversarial suites assert on the spike's own class (`pytest.raises(verify.EvidenceFailure, ...)`); a shared base class raised from shared helpers would change what those cases catch, which this ticket's constraint rules out. So it trades about seventy duplicated lines for about the same number of wrappers, adds a cross-directory `importlib` load, and makes the sole custodian of evidence nothing recompiles no longer readable in one file.

**Fact — the third custodian is not a third copy.** `spikes/extensions/run.py`'s `verify_visibility_evidence` iterates every record under `results/` and requires the running compiler to be among those they name; both shape spikes require exactly one record and compare one channel. Its `sole_measurement` returns the first of many by design where `sole_record` refuses a second. So the premise that the extensions harness holds a copy to be folded in does not hold, and the original trigger — "a third spike needs a retained-claims record" — was already spent without firing.

**Defect found and fixed — the copy had diverged where it counts.** `verify_evidence.py`'s `verify_failing_case` carried a comment stating that both fragment lists "are required rather than defaulted. A claim that lost its `required_fragments` would otherwise keep passing while asserting nothing", but the check only rejected a *missing* key: `[]` is a list of strings, so an emptied `required_fragments` or `diagnostic_codes` satisfied every structural predicate and asserted nothing. The gated sibling already refused that through `required_fragment_list`. That predicate is now in the off-pin verifier too, applied to `diagnostic_codes` and `required_fragments` and deliberately not to `forbidden_fragments` — two off-pin cases legitimately forbid nothing, because they share no diagnostic code with another case, which was confirmed against the record before tightening.

**Constraint satisfied, with one message reworded.** Both suites still refuse everything they refused, for the same reasons: `spikes/shapes` now passes 59 cases, up from 51, the eight new ones being two adversarial cases here plus the six the two suites already gained. `spikes/shapes/shape-evidence/test_shape_evidence_record.py` is 25 cases, up from 23 — the two additions are `empty_claim_field("diagnostic_codes")` and `empty_claim_field("required_fragments")`, the quieter half of the existing `drop_claim_field` pair. One existing expectation was reworded, not weakened: the dropped-codes case refused with "records no diagnostic code list" and now refuses with "records no diagnostic_codes list", because the shared predicate derives the wording from the key. `spikes/shapes/nightly-dependent-static-shapes` is unchanged at 31.

**Also corrected.** `spikes/shapes/shape-evidence/README.md` claimed "twenty cases that each corrupt a copy"; the count was 22 before this ticket and is 24 now. It is stated as twenty-four.

**The trigger, restated so it can fire.** Not "a third spike appears" — that already happened and was the wrong signal. Instead: a *rule* about what a claim must assert, rather than a file-reading helper, has to be changed in more than one verifier at once. That has now happened exactly once, in this ticket, with `required_fragment_list`. A second occurrence is the evidence that the seven-flag cost is worth paying, and the restated trigger is recorded in both module docstrings alongside the decision.
