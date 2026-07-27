---
id: remove-the-planner-s-per-plan-string-and-clone-churn
title: Remove the planner's per-plan string and clone churn
status: done
priority: p2
dependencies: []
related: [remove-the-remaining-duplicate-work-in-the-planner]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [performance, compiler]
---
Dispatched from a fresh profile of the compile loop. Compile is 882 us for a 5-operation program; the remaining self time is diffuse memmove/memcmp/allocation rather than one hot function.

## Hypotheses to test (none verified at dispatch)

- `SelectedPlanIdentity::label` allocates a String per plan; callers may only compare.
- The O(P^2) Pareto scan may run twice (selection then `verify_portfolio`).
- `pipeline.rs` ~1092 may clone whole covers into `sources`.
- `label()`-per-comparison at O(refused x P) around `pipeline.rs:1090`/`:1103`.

## Constraints

- Artifact and plan/portfolio identity bytes unchanged.
- `verify_portfolio` is a boundary check for externally supplied portfolios; make it cheaper, do not weaken it.
- Report min-of-200, never mean.

## Closes when

Profile-directed changes are landed and measured before/after, or each hypothesis is explicitly refuted with the measurement that refuted it.
