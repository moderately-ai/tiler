---
id: consolidate-private-compiler-target-concerns
title: Consolidate private compiler target concerns
status: todo
priority: p2
dependencies: [express-metal-honourability-in-the-shared-form]
related: [prototype-public-compiler-api, decide-per-dtype-dispatchability-as-a-target-capability]
scopes: [implementation/compiler]
shared_scopes: []
paths: []
tags: [refactor, compiler, target-profiles, progressive-disclosure]
---
Make the compiler's target-description and assessment boundary visible without
putting hardware facts into the logical graph.

Target requests, profile description, feasibility, honourability, and
assessment are currently spread across several top-level compiler modules. The
shared honourability ownership decision must land first so this refactor does
not assume where the target-neutral vocabulary belongs.

## User-visible outcome

Create one shallow private target cluster whose files own profile description,
hard feasibility, numerical honourability, and assessment. Preserve the
distinction between feasibility and cost and preserve the reviewed public
session facade.

This is organization, not a new target IR, and not permission to make the
compiler depend on Metal-specific types.

## Closes when

Target-related dependency direction is visible from one private module root,
logical IR remains target-independent, feasibility remains distinct from cost,
and the full gate passes without public-path changes.
