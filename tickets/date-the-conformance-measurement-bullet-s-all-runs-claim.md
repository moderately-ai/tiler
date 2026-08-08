---
id: date-the-conformance-measurement-bullet-s-all-runs-claim
title: Date the conformance measurement bullet's all-runs claim
status: todo
priority: p3
dependencies: []
related: [refresh-the-device-free-test-floor-s-prose-census]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: []
---

`docs/status.md`'s device-execution Measurement bullet says `crates/tiler-conformance` "was admitted on 2026-08-07 and every one of its runs landed the same day." The runs remain the 2026-08-07 population, but later crate edits mean the undated sentence no longer identifies its historical boundary.

## Facts, read at `0339d28e`

**Fact.** The source-safe anchor `every one of its runs landed the same day` is live in `docs/status.md`'s `The conformance crate's three verticals` bullet. It is a navigation record, not a conformance-crate comment.

**Fact.** `f519c695` and `fe282f1e` add no `require_or_report` call site; the latter adds the device-free `the_serial_sum_identity_crosses_the_shared_opaque_bound_at_the_second_contributor` test. Thus the statement remains true of the measured-run population but is imprecise as an undated account of the crate.

## Outcome

Read the acceptance and construction history, then retain the historical wording in a dated correction beside the status bullet. Name the measured-entry construction rather than inventing a new hand-maintained count, and say that later grep hits for the retired wording land in the note. Do not change `crates/tiler-conformance` on this ticket.

## Relationship

[`refresh-the-device-free-test-floor-s-prose-census`](refresh-the-device-free-test-floor-s-prose-census.md) found this independent navigation defect while repairing the floor comment. The tickets are related, not dependent: the source-census repair can complete without a status-page edit.
