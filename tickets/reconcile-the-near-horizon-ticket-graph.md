---
id: reconcile-the-near-horizon-ticket-graph
title: Reconcile the near-horizon ticket graph
status: done
priority: p1
dependencies: []
related: []
scopes: []
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

The dependency-ready horizon describes the code and accepted contracts that exist after the Metal launch-limit vertical, exposes every prerequisite needed for the next implementation slices, and contains no shortcut whose cheaper shape loses correctness, identity, provenance, performance evidence, or a required public review.

## Work

Reconcile the next dependency-ordered implementation horizon against current construction sites, accepted ADRs, vendored primary evidence, and merged identity/schema authorities. Correct stale premises in place; split tickets whose independent proof, backend realization, calibration, or public-boundary work cannot honestly close together; add missing dependencies and related edges; and create bounded follow-ups where a ticket currently assumes an authority or representation that does not exist.

Keep every decision already derived from correctness as an outcome rather than reopening it. Record genuine unresolved public boundaries as exact future review gates, and preserve historical evidence as historical rather than rewriting it into a current source claim. Do not change production code or mark implementation work complete in this graph-maintenance ticket.

## Required evidence

Audit at least the next twenty dependency-ordered tickets from `tkt lanes`, inspect every ticket changed in full, reproduce each corrected negative or absence claim against the current tree, and run `tkt lint`, `git diff --check`, and `tkt guard` against the exact branch base. An independent reviewer must verify that the resulting graph has no dependency inversion, deadlock, hidden public boundary, stale version authority, or combined ticket whose closing evidence cannot be produced on one tree.

## Closes when

Every audited ticket has a current user-visible outcome, actionable implementation keys, required evidence with non-vacuity expectations, and graph maintenance; newly exposed prerequisites are filed and linked; eliminated alternatives state why they fail; the board checks pass; the exact commit is independently reviewed; and the corrected graph is integrated before another implementation ticket is dispatched.

## Graph maintenance

This is additive maintenance over `project/tickets` only. Close it after the corrected graph lands; implementation tickets created or repaired by the sweep remain open and become ordinary dependency-ordered work.
