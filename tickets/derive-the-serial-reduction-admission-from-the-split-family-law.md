---
id: derive-the-serial-reduction-admission-from-the-split-family-law
title: Derive the serial reduction admission from the split family law
status: todo
priority: p3
dependencies: []
related: []
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

A reduction family states its contributor tensor, empty-domain obligation, and reassociation consumption in one place read by all three topologies; a fifth family becomes one `SplitFamily` arm instead of one match arm per topology.

## Why this exists (vocabulary audit 2026-08-06; the auditor hand-verified no serial/parallel divergence exists today — this is prophylactic, and saying so is the point)

`SplitFamily`'s own doc states the law ("a family admitted by one admission and not the other would otherwise be a difference nobody states"); `multi_pass_family` and `cooperative_family` implement it; the four serial arms under `fn verify_access_and_semantics` do not — nine byte-identical conjuncts repeated, three genuinely differing facts inlined, and the empty-domain rule spelled a fourth time outside `empty_domain_is_satisfied` whose doc claims coverage "at each admission". VERIFY the nine-conjunct claim by reading the four arms before starting.

## Boundaries

`consumes_reassociation` has no serial meaning and must not acquire one (a serial fold spends nothing). The extrema non-emptiness precondition folds into `empty_domain_is_satisfied`, not a fourth rule. No admission widens or narrows — bit-identical admitted sets, pinned by the unchanged canonical-identity tests. All types private; no boundary, no encoder.

## Closes when

One shared conjunction with a per-family derivation function; a test asserting per-family agreement across the three admissions, watched failing under a perturbed serial read tensor; identity pins unchanged.
