---
id: derive-the-serial-reduction-admission-from-the-split-family-law
title: Derive the serial reduction admission from the split family law
status: todo
priority: p3
dependencies: []
related: [admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary]
scopes: [implementation/ir]
shared_scopes: [project/tickets]
paths: []
tags: []
---
## User-visible outcome

A reduction family states its contributor tensor, empty-domain obligation, and reassociation consumption in one place read by all three topologies; a fifth family becomes one `SplitFamily` arm instead of one match arm per topology.

## Why this exists (vocabulary audit 2026-08-06; the auditor hand-verified no serial/parallel divergence exists today — this is prophylactic, and saying so is the point)

`SplitFamily`'s own doc states the law ("a family admitted by one admission and not the other would otherwise be a difference nobody states"); `multi_pass_family` and `cooperative_family` implement it; the **five** serial fold arms under `fn verify_access_and_semantics` do not — nine byte-identical conjuncts repeated across all five, three genuinely differing family facts inlined, and the empty-domain rule spelled a fourth time outside `empty_domain_is_satisfied` whose doc claims coverage "at each admission". The four multi-topology families (sum, fused, squared, maximum) are the `SplitFamily` population; `SquaredSerialSumThenEpilogue` is a fifth serial-only arm that repeats the nine shared conjuncts and adds epilogue residuals while both parallel tables answer `None`. VERIFY the nine-conjunct claim by reading the five arms before starting; any shared derivation must either carry ThenEpilogue as serial-only post-checks or leave it as an explicit residual arm rather than silently drop it.

## Boundaries

`consumes_reassociation` has no serial meaning and must not acquire one (a serial fold spends nothing). The extrema non-emptiness precondition folds into `empty_domain_is_satisfied`, not a fourth rule. No admission widens or narrows — bit-identical admitted sets, pinned by the unchanged canonical-identity tests. All types private; no boundary, no encoder.

## Graph repair — 2026-08-10

Related [`admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary`](admit-a-fold-over-any-declared-input-in-the-scheduled-region-vocabulary.md): concurrent open work on the same serial arms / `implementation/ir` surface; it widens DeclaredDomain / fused FIRST exactness and is the event that would turn this prophylaxis into a live divergence risk if done without shared derivation. No hard dependency either way — sequencing hygiene only.

**Census correction.** Prior prose said "four serial arms"; at base `c99ac54950f2` the serial match has five fold arms (`serial_fold_families` asserts `families.len() == 5`). Production comment "at all four" above the serial match is the same stale census and is not authority.

## Closes when

One shared conjunction with a per-family derivation function; a test asserting per-family agreement across the three admissions, watched failing under a perturbed serial read tensor; identity pins unchanged; the five-family unit population still forces every serial fold arm.
