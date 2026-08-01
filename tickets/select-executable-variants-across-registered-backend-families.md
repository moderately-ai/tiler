---
id: select-executable-variants-across-registered-backend-families
title: Select executable variants across registered backend families
status: todo
priority: p1
dependencies: [produce-a-custom-backend-payload-through-the-build-orchestrator, route-a-custom-backend-through-an-independently-selected-adapter]
related: [prototype-complete-physical-plan-selection, promote-artifact-family-selection-for-the-frontend]
scopes: [implementation/runtime, implementation/artifact, implementation/compiler, contracts/artifacts, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, selection, runtime]
---
## User-visible outcome

A multi-backend artifact filters variants by installed executable backend/representation and live target eligibility before applying routing policy, so an incompatible first variant cannot hide a later valid CPU, Metal, or custom alternative.

## Implementation keys

- Replace the current effective order—first true guard, then compatibility—with adapter/profile eligibility, applicability guards, policy comparison, and one-way commit.
- Preserve artifact stable-priority semantics where requested while making ineligible variants non-candidates rather than terminal mismatches.
- Keep compile-time plan cost selection separate from runtime routing policy; runtime must not invoke the optimizer or invent unrecorded costs.
- Define typed outcomes for missing adapter, duplicate adapter authority, unsupported representation, profile mismatch, no guard match, and no eligible variant.
- Verify complete semantic/numerical equivalence or explicit fallback coverage before packaging alternatives.
- Ensure one variant may legitimately use several payloads without treating that as multi-device execution.
- Perturb provider registration, variant ordering, eligibility, guard results, and commit timing; prove the old algorithm fails the cross-backend fixture.
- Present the exact routing-policy and public result boundary to Tom.

## Closes when

An artifact containing at least two backend families selects a later compatible variant when an earlier one is unavailable, preserves stable priority among eligible variants, fails closed when none qualify, and all identity, explain, targeted, and full-gate checks pass.

## Graph maintenance

- Do not conflate backend-family selection with frontend artifact-family delivery; keep the existing frontend ticket independent.
- Feed the accepted eligibility model into the public composition/policy facade.
- Keep multi-device/sharding deferred: this ticket selects one executable route, not several devices.
