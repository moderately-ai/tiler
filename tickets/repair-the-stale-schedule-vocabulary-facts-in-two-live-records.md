---
id: repair-the-stale-schedule-vocabulary-facts-in-two-live-records
title: Repair the stale schedule-vocabulary Facts in two live records
status: in-progress
priority: p3
dependencies: []
related: [point-the-bare-builder-path-mentions-at-the-split-modules]
scopes: [contracts/decisions, research/scheduling]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, doc-drift]
claimed_from: todo
assignee: worker-schedule-facts
lease_expires_at: 1787157651
---
## User-visible outcome

ADR 0097 and `docs/research/scheduling/cpu-vector-lane-tier.md` stop asserting Facts the tree refutes: the ADR's named test and pin resolve again, and the lane-tier record's tail-policy and padding Facts state what the schedule vocabulary now admits.

## Why this exists — filed 2026-08-19 from the bare-path lane's out-of-fence report

Both finds verified by that lane with commands, recorded not repaired because they sat outside its navigation fence:

- **ADR 0097** names `the_staging_relation_step_moves_only_the_domain_separator` and `STRICT_F32_REGION_IDENTITY_HEX_V4`; both grep empty under `crates/`. The schedule domain has stepped to `v7`; the retained comparison is `STRICT_F32_REGION_IDENTITY_HEX_V6` and the one-step test is `the_elementary_dimension_step_moves_domain_and_payload()` (`crates/tiler-ir/src/schedule/builder/tests.rs`). Repair with a dated correction preserving the record's original claim as history.
- **`cpu-vector-lane-tier.md`** carries three stale Facts (the lane already added a dated flag naming them): the per-topology reassociation claim is now per-family (`family.consumes_reassociation`); `TailPolicy` now has `Exact` **and** `Predicated`; identity padding **is** admitted via `ContributorCoverage::IdentityPadded` with derived two-sided neutrality in `crates/tiler-ir/src/schedule/builder/coverage.rs`. Each needs a dated correction stating the current vocabulary with grep-verified anchors. `scheduled-region-model.md:496` repeats the one-variant `TailPolicy` claim — include it after re-reading its context.

## Required work

Read each cited source in full at the working base before writing; dated corrections in each document's own convention, never silent restatement; every anchor grep-verified. Gates: `make citations`, `tkt lint`, `git diff --check`, `tkt guard`.

## Closes when

Both records' stale Facts carry dated corrections whose anchors resolve, and no named test or pin in either record greps empty against the tree.
