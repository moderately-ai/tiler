---
id: select-executable-variants-across-registered-backend-families
title: Select executable variants across registered backend families
status: in-progress
priority: p1
dependencies: [produce-a-custom-backend-payload-through-the-build-orchestrator, route-a-custom-backend-through-an-independently-selected-adapter]
related: [prototype-complete-physical-plan-selection, promote-artifact-family-selection-for-the-frontend]
scopes: [implementation/runtime, implementation/artifact, implementation/compiler, contracts/artifacts, contracts/integrations]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, selection, runtime]
claimed_from: todo
assignee: worker-variant-sel
lease_expires_at: 1785565152
---
## User-visible outcome

A multi-backend artifact filters variants by installed executable backend/representation and live target eligibility before applying routing policy, so an incompatible first variant cannot hide a later valid CPU, Metal, or custom alternative.

## Implementation keys

- **Corrected 2026-08-01 against accepted [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), which this ticket predates:** there is no adapter registry — eligibility derives from the loading host's *stated* `ExecutionEnvironment` (backend/representation compared as a pair, profile classified) exactly as the loader already does for one variant; this ticket widens that to filter a multi-family portfolio. Replace the current effective order — first true guard, then compatibility — with host-environment eligibility, applicability guards, policy comparison, and one-way commit.
- Preserve artifact stable-priority semantics where requested while making ineligible variants non-candidates rather than terminal mismatches.
- Keep compile-time plan cost selection separate from runtime routing policy; runtime must not invoke the optimizer or invent unrecorded costs.
- Define typed outcomes for unsupported representation, profile mismatch, no guard match, and no eligible variant. (The registry-era outcomes this key named — missing adapter, duplicate adapter authority — do not exist under the accepted no-registry model; a host that cannot execute a family simply declares an environment no variant of that family matches.)
- Verify complete semantic/numerical equivalence or explicit fallback coverage before packaging alternatives.
- Ensure one variant may legitimately use several payloads without treating that as multi-device execution.
- Perturb the host's stated environment, variant ordering, eligibility, guard results, and commit timing; prove the old algorithm fails the cross-backend fixture.
- Present the exact routing-policy and public result boundary to Tom.

## Closes when

An artifact containing at least two backend families selects a later compatible variant when an earlier one is unavailable, preserves stable priority among eligible variants, fails closed when none qualify, and all identity, explain, targeted, and full-gate checks pass.

## Graph maintenance

- Do not conflate backend-family selection with frontend artifact-family delivery; keep the existing frontend ticket independent.
- Feed the accepted eligibility model into the public composition/policy facade.
- Keep multi-device/sharding deferred: this ticket selects one executable route, not several devices.
