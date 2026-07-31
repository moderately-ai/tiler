---
id: prototype-a-forkless-custom-metal-physical-provider
title: Prototype a forkless custom Metal physical provider
status: in-progress
priority: p1
dependencies: []
related: [draft-public-extension-seam-ownership-adr, prototype-complete-physical-plan-selection, implement-opaque-physical-call-providers]
scopes: [research/extensions]
shared_scopes: [research/program-planning, project/tickets]
paths: []
tags: [backend-providers, pluggability, metal, spike]
claimed_from: todo
assignee: loop-prototype-a-
lease_expires_at: 1785517249
---
## User-visible outcome

A retained executable spike demonstrates whether a separately authored, statically linked provider can contribute one specialized Metal physical implementation alongside Tiler's governed provider without forking or replacing `tiler-metal`.

## Why this slice exists

The internal physical frontier is additive, but `PhysicalImplementationProvider` is crate-private and no backend-defined provider reaches the ordinary compile path. A custom provider that must replace the entire backend would not meet the fork-avoidance goal.

## Implementation keys

- Keep the spike private and bounded; it is evidence for a later public boundary, not an implicit promotion.
- Choose one real supported region for which both the governed provider and the custom provider can offer correct implementations.
- Give the custom provider a distinct stable identity and output-affecting revision.
- Drive both alternatives through ordinary schedule, feasibility, structured-KIR, and selected-plan verification; the custom provider may propose but may not stamp provenance, resource requirements, or boundary guarantees.
- Demonstrate that registration order does not select the winner, lowering-authority contention remains an error, and physical alternatives remain additive.
- Reuse standard Metal emission and runtime behavior rather than copying it; record any interface that prevents this reuse.
- Perturb the provider output into a malformed schedule or mismatched region and observe the verifier fail.
- Preserve the spike harness, exact invocation, inputs, and result fixture under `spikes/`.
- Do not edit production crates in this spike. File any evidence-backed production blocker as a separate ticket with its own scope and public-boundary review where required.

## Closes when

The spike proves or falsifies partial Metal-provider composition, records the exact private surfaces it needed, distinguishes verified guarantees from trusted semantic-equivalence claims, and leaves no production API or crate admission behind.

## Graph maintenance

- Feed the measured interface and reuse requirements into `specify-the-consumer-neutral-backend-provider-composition-contract`.
- If reuse fails because Metal emission or runtime ownership is inseparable, file the smallest evidence-backed split ticket rather than widening this spike.
- Do not treat opaque physical-call registration as backend-provider registration; keep the tickets related but distinct.
