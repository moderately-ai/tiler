---
id: express-the-typed-backend-family-selection-policy
title: Express the typed backend-family selection policy
status: closed
closed_reason: wontdo
closed_note: Tom requires one explicitly selected backend environment per routing attempt and forbids automatic cross-family fallback.
priority: p1
dependencies: [decide-whether-a-loading-host-may-state-several-backend-families]
related: [expose-explicit-backend-provider-and-selection-policy-composition, select-executable-variants-across-registered-backend-families, exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio]
scopes: [implementation/runtime, contracts/artifacts, contracts/integrations, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, runtime, selection]
---
## User-visible outcome

A consumer that links more than one backend states allowed, required, or fallback-only backend-family policy as typed input, and a policy permitting no executable route is refused before any routing work rather than discovered as an absent variant.

## Closed — 2026-08-11

Tom rejected this outcome in the T3 Code orchestration conversation. A user must explicitly select one backend approach for each routing attempt; `preflight` or `prepare` validates that exact singular [`ExecutionEnvironment`](../crates/tiler-runtime/src/load/host.rs), and the loader never silently retries another backend family. A caller may make a separate explicit attempt after a preflight refusal and before any commit, but there is no allowed/required/fallback-only cross-family vocabulary to implement.

This is `wontdo`, not delivered work. The existing within-family stable-priority variant selection remains; what is rejected is automatic selection or fallback **between** backend families. The accepted decision and provenance are recorded in [`decide-whether-a-loading-host-may-state-several-backend-families`](decide-whether-a-loading-host-may-state-several-backend-families.md).

## Why this is separate from its parent

[`expose-explicit-backend-provider-and-selection-policy-composition`](expose-explicit-backend-provider-and-selection-policy-composition.md) discharged every one of its keys that the accepted model still admits: the compiler-side composition seams landed, the governed-cost-identity rule landed with its `compile_fail` evidence, and two of its keys turned out to name mechanisms [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) eliminated. This is the one key that survives correction, and it is parked rather than ready because the layer that owns it is [`decide-whether-a-loading-host-may-state-several-backend-families`](decide-whether-a-loading-host-may-state-several-backend-families.md)'s to fix.

## Implementation keys

- Build against whichever host model Tom accepts. Under Option A the vocabulary and its refusal live in `tiler-runtime`; under Option B they live in the consumer facade and this ticket must first add `implementation/frontend`, which it deliberately does not yet declare.
- Keep the policy **cost-side and the eligibility filter feasibility-side**, as separate stages with separate refusals. A family the host cannot execute and a family the consumer declined are two findings with two repairs, and `VariantIneligibility` must not grow a policy class — that is the conflation the split exists to prevent.
- Let the policy only *restrict* families the host already stated, refusing at construction if it names another. Handing the consumer a second way to say which families exist would create a second derivation of an answer the host declaration already gives.
- Preserve the producer's `RoutingPolicy::StablePriority` order as the ranking **within** whatever the policy permits. The consumer partitions families into preference tiers; the producer orders plans inside a tier. Two authorities ordering along one axis is the failure mode; along different axes it is not.
- Keep an eligible variant's unanswerable applicability guard aborting the walk rather than falling through to a lower tier. Falling through would substitute a plan the consumer ranked lower because the caller bound too little, which is the plan substitution `select_variant` already documents.
- Do not put the policy in the artifact beside `RoutingPolicy`. It is a consumer deployment fact, not a producer one; encoding it would fold a consumer's preference into artifact identity and move every pinned digest for a value the producer does not hold.
- Add the Metal-or-CPU compile-pass example the parent ticket's example key still owes, and the no-valid-route refusal beside it.
- Perturb each guarded property separately: the construction-time refusal, the tier ordering, the within-tier producer ordering, and the unanswerable-guard abort. Show a control for each.
- Present the exact public surface to Tom under ADR 0075 rather than self-accepting it.

## Closes when

A consumer states each of allowed, required, and fallback-only against a portfolio it can partly execute and gets the routed family the policy names; a policy permitting nothing the host stated is refused at construction with its own typed class; the producer's stable priority is unchanged within a tier; every perturbation reddens the property it guards and no other; and targeted plus full gates pass.

## Graph maintenance

- Feed the accepted vocabulary into [`exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio`](exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio.md), which tests it end to end and which currently depends on the parent for a vocabulary the parent no longer delivers.
- Keep multi-device and sharding deferred: a policy chooses one route among families, never several devices.
