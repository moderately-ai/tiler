---
id: exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio
title: Exercise standard Metal, custom Metal, and CPU providers in one portfolio
status: todo
priority: p1
dependencies: [expose-explicit-backend-provider-and-selection-policy-composition, express-the-typed-backend-family-selection-policy, join-build-time-producers-to-runtime-adapters-through-artifact-identity]
related: [prototype-inline-aot-integration-proof, prototype-metal-runtime-proof]
scopes: [research/runtime, research/extensions, research/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, integration, metal, cpu]
---
## User-visible outcome

One retained end-to-end proof composes standard Metal, a forkless custom Metal specialization, and a bounded CPU backend, packages their valid alternatives, selects an executable route from installed adapters and policy, and matches the independent reference result.

## Implementation keys

- Use one semantic program that exercises the custom Metal candidate while retaining a valid standard Metal and CPU route.
- Compile through the public provider composition facade and record exact physical/provider selection provenance.
- Produce backend payloads without duplicating semantic meaning or runtime adapter logic.
- Package a complete portfolio whose variants and payloads carry independent backend, representation, target, and compilation identities.
- Run CPU on every host; run Metal legs only on eligible measured hosts and report explicit unavailability rather than silently passing.
- Test standard-Metal-only, custom-Metal-preferred, CPU-only, Metal-or-CPU, missing-adapter, incompatible-profile, and no-valid-route policies.
- Prove a custom provider can be removed without forking or changing standard Metal.
- Compare exact outputs and observable materialization boundaries with `tiler-reference`.
- Perturb every selection and routing check and watch the proof fail.
- Retain the executable spike under `spikes/runtime/backend-provider-portfolio` with its exact manual invocation, inputs, and result fixture. The spike command proves this vertical; `make full` independently proves the ordinary workspace remains green because no root gate reaches `spikes/`.
- File any production change the spike discovers as a separate evidence-backed ticket rather than implementing it inside this integration proof.

## Closes when

The retained fixture's recorded manual command demonstrates forkless partial extension and genuinely different backend execution through the same accepted composition model, all unavailable/invalid cases fail closed and explainably, exact identities are rebaselined on the merged tree, and an independent `make full` passes.

## Graph maintenance

- Mark the CPU result only as the bounded profile the fixture exercises; production CPU/SIMD breadth remains separate.
- Feed the complete positive/negative population into the reusable conformance suite.
- Do not activate multi-device/sharding: each run commits to one route and one live execution context.
