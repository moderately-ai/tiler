---
id: reopen-the-joint-permission-question-when-a-blocker-falls
title: Reopen the joint permission question when a blocker falls
status: deferred
priority: p3
dependencies: []
related: [reassess-the-distributivity-decline-against-the-online-softmax-rescaling-caller, derive-the-value-precondition-the-online-softmax-bound-needs-for-its-subnormal-clause, admit-a-handed-value-with-more-than-one-reader-in-the-region-sequence, close-the-eps-exp-open-axis-in-the-rule-object-record]
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: [decision, numerics, deferred]
---
## What this holds

[ADR 0095](../docs/decisions/0095-decline-a-distributivity-permission.md)'s second reopening condition — added by its 2026-08-06 reaffirmation, that the distributivity and elementary-identity permissions be considered together when a rewrite rule with a derived bound instantiable at a schedulable fold shape is ready to consume them — **fired on 2026-08-06**: all three prerequisites hold, per [the rule-object record](../docs/research/numerics/online-softmax-rule-object.md)'s Part 6 re-derivation under [`close-the-eps-exp-open-axis-in-the-rule-object-record`](close-the-eps-exp-open-axis-in-the-rule-object-record.md).

**Tom decided on 2026-08-06, at the live session's decision round via the coordinator's presentation (AskUserQuestion): hold, with a trigger.** Grounds, as presented and accepted: readiness is not reachability. Even with both permissions admitted, the rewrite stays refused — obligation 1 refuses on `SOFTMAX_F32_FACT_SUBNORMALS` independently of any permission, and obligation 3 wants a merge topology no schedule type carries — so reopening today would change the refusal, not the outcome.

## Trigger

Either blocker falling makes the joint question worth presenting again:

1. **The subnormal value precondition lands** — [`derive-the-value-precondition-the-online-softmax-bound-needs-for-its-subnormal-clause`](derive-the-value-precondition-the-online-softmax-bound-needs-for-its-subnormal-clause.md) closes with a dischargeable precondition, or
2. **A merge topology becomes expressible** — a schedule type carries the `(m, d)` pair-merge topology the rule-object record's obligation 3 names.

On either, present the joint admission question to Tom with the reassessment packet's outcome-4 material and the then-current obligation tally.

## Trigger check log

- 2026-08-06 — **not fired.** Blocker 1: `grep -m1 '^status:' tickets/derive-the-value-precondition-the-online-softmax-bound-needs-for-its-subnormal-clause.md` returns `status: todo`. Blocker 2: the rule-object record's obligation 3 stands as derived; no schedule type carries the merge topology.
- 2026-08-09 — **not fired; the first blocker's old status is retired.** `derive-the-value-precondition-the-online-softmax-bound-needs-for-its-subnormal-clause` is now `deferred`, not `todo`, because no governed precondition vocabulary can discharge the subnormal clause yet. No schedule type carries the required `(m, d)` pair-merge topology either. Both substantive blockers therefore remain even though the ticket-state spelling changed.
