---
id: audit-dead-code-admissions-after-public-boundary-promotions
title: Audit dead-code admissions after public-boundary promotions
status: deferred
priority: p3
dependencies: [promote-the-symbolic-index-profile-to-a-public-boundary, promote-the-metal-aot-compilation-identity, expose-the-governed-fact-field-vocabulary, expose-the-numerical-contract-preference-list, wire-the-delivered-realization-record-into-the-artifact]
related: []
scopes: [implementation/ir, implementation/compiler, implementation/artifact, implementation/metal-aot]
shared_scopes: [project/tickets]
paths: []
tags: [maintenance, lints, public-boundary, deferred]
---
Re-audit production `dead_code` admissions after the current public-boundary
promotion wave makes the intended producers and consumers reachable.

The whole-file admissions observed by the production audit are no longer in the
current tree. Remaining admissions are narrow, so this ticket must not recreate
the stale claim that an entire production file is suppressed.

## Outcome

Remove an admission when its item is now used or public. Keep a private
reservation only at the narrowest item or submodule whose missing producer or
consumer is real, with a reason naming that boundary and the trigger that will
reopen it. Do not add artificial call sites merely to satisfy the lint.

## Closes when

Every production `dead_code` admission is either gone or justified against a
current construction/consumer search, no whole-file admission has returned,
and the full gate passes.
