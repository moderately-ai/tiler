Ticket: scope-the-standalone-extrema-and-clamp-families
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-standalone-extrema-and-clamp-families/eaa27984d262_c99ac54950f2.md
Pre-edit content hash (from ledger): eaa27984d262a80ca0caddf25c283d1b52d37c59799a6b189d21240b2ce5cabb
Post-edit content hash: 7f9b85bdfed4f89e8f700506e21e7784262d7c13fcfc0443b3f69bd55f906263

Changes applied:
  - Replaced the FALSE Maximum identity claim in the work body; live text no longer asserts "no binary32 value is an identity for Maximum". Dated **Correction — 2026-08-10** records that `StrictSerialMaximum` omits `empty_identity_bits` because the empty-domain *result* is undeclared, while `-inf` is observably neutral padding under ADRs 0022/0025 (schedule/model.rs StrictSerialMaximum docs).
  - Non-goal parallel-topology owner: past tense; names `admit-a-parallel-topology-for-the-identity-less-extrema-fold` as `status: done` (no "live owner").
  - Resolved O-19 vs O-39 dual ownership on this ticket: O-19 owns F-10 elementwise + clamp five identities; standalone extrema reduction under F-28 is O-39 (`scope-the-monoid-reducers-beyond-the-strict-sum`). Updated Outcome, Activation trigger, work body, non-goals, Closes when, and Graph maintenance; dated correction quotes retired dual-form Graph claim.
  - Optional related: added `scope-the-monoid-reducers-beyond-the-strict-sum` for bidirectional ownership visibility (reverse edge already present on that ticket).
  - Trigger check log: dated the 2026-08-05 "46 / eighteen" census as historical; added 2026-08-10 **not fired** recheck (47 unique `tiler::…@N` keys; no minimum/maximum/clamp/relu family keys).

Optional items skipped (with reason):
  - none; the optional monoid-reducers related edge was cheap graph hygiene and was applied.

Residuals not applied (docs/crates/new tickets/authority):
  - `tickets/scope-the-monoid-reducers-beyond-the-strict-sum.md`: same historical identity falsehood may still appear in older log wording; boundary alignment prose already matches this repair's O-19/O-39 split on the monoid ticket, but that file was out of this worker's single-ticket edit set.
  - `docs/research/semantic-graph/operation-family-delivery-graph.md`: O-19/O-39 form-ownership one-sentence consistency if any row still reads as O-19 owning both elementwise and reduction forms (Class C docs residual).
  - Optionally `docs/decisions/0022-reduction-identities-and-initial-values.md` implementation-boundary stale "no binary32 value is neutral" sentence (carrier-owned; docs/decisions out of ticket scopes).
  - Product activation remains deferred; no OpKeys or emission work in this wave.

Verification:
  - files read:
    - tickets/scope-the-standalone-extrema-and-clamp-families.md (full, pre and post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-standalone-extrema-and-clamp-families/eaa27984d262_c99ac54950f2.md (full)
    - tickets/scope-the-monoid-reducers-beyond-the-strict-sum.md (frontmatter through Graph maintenance; O-39 non-goal already claims only reduction form)
    - tickets/admit-a-parallel-topology-for-the-identity-less-extrema-fold.md (status: done)
    - crates/tiler-ir/src/schedule/model.rs StrictSerialMaximum empty-domain / `-inf` docs
    - docs/research/semantic-graph/operation-family-delivery-graph.md (O-19 F-10 / O-39 F-28 rows)
    - docs/roadmap.md extrema matrix trigger cell
  - checks:
    - `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` → 47 unique keys; no minimum/maximum/clamp/relu family keys
    - live body no longer asserts present-tense "no binary32 value is an identity for Maximum" or "live owner" (retired wording only inside dated corrections)
    - `shasum -a 256 tickets/scope-the-standalone-extrema-and-clamp-families.md` → 7f9b85bdfed4f89e8f700506e21e7784262d7c13fcfc0443b3f69bd55f906263

Recommended next ledger state:
  integrated
