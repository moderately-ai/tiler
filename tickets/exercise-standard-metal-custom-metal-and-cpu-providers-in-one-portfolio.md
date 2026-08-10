---
id: exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio
title: Exercise standard Metal, custom Metal, and CPU providers in one portfolio
status: todo
priority: p1
dependencies: [expose-explicit-backend-provider-and-selection-policy-composition, express-the-typed-backend-family-selection-policy, join-build-time-producers-to-runtime-adapters-through-artifact-identity]
related: [prototype-inline-aot-integration-proof, prototype-metal-runtime-proof, decide-whether-a-loading-host-may-state-several-backend-families, select-executable-variants-across-registered-backend-families, publish-the-backend-provider-conformance-suite]
scopes: [research/runtime, research/extensions, research/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, integration, metal, cpu]
---
## User-visible outcome

One retained end-to-end proof composes standard Metal, a forkless custom Metal specialization, and a bounded CPU backend, packages their valid alternatives, selects an executable route from installed adapters and policy, and matches the independent reference result.

## Implementation keys

**Corrected 2026-08-10 against accepted [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 4 (no `BackendProvider` / no provider bundle) and the [`select-executable-variants-across-registered-backend-families`](select-executable-variants-across-registered-backend-families.md) Outcome (no-registry routing).** Custom Metal is a physical-provider row claim on the Metal family, not a third backend family: the portfolio story is two physical authorities under Metal plus one CPU family (and their adapters/payloads), under one semantic program. Family-policy vocabulary is owned by [`express-the-typed-backend-family-selection-policy`](express-the-typed-backend-family-selection-policy.md) (blocked on [`decide-whether-a-loading-host-may-state-several-backend-families`](decide-whether-a-loading-host-may-state-several-backend-families.md)); this ticket exercises those seams end to end once they land.

- Use one semantic program that exercises the custom Metal candidate while retaining a valid standard Metal and CPU route.
- Drive the accepted per-responsibility composition path end to end — compile: `CompileRequest::with_physical_providers` / offered+selected physical provenance; build: `assemble_plan_artifact` / payload publish; runtime: host `ExecutionEnvironment` + `route_with_adapter`; family policy from express once accepted — and record exact physical/provider selection provenance. ~~Compile through the public provider composition facade~~ **False name relative to ADR 0090 item 4:** there is no `BackendProvider`, no provider bundle, and no single facade type; composition is the per-row seams above.
- Produce backend payloads without duplicating semantic meaning or runtime adapter logic.
- Package a complete portfolio whose members vary backend, representation, payload profile, and compilation subjects independently while sharing one assessed variant-level `TargetProfileRef` (and one numerical contract) pinned by artifact `check_subject`. ~~independent backend, representation, target, and compilation identities~~ **Imprecise on target:** variant-level target is one subject for the portfolio; independence holds for payload-side profile / backend / representation / compilation, not for a second assessed variant target.
- Run CPU on every host; run Metal legs only on eligible measured hosts and report explicit unavailability rather than silently passing.
- Exercise two axes, not one flat policy list: (i) **within Metal** — governed vs installed custom physical provider selection / preference among plan alternatives; (ii) **across families** — standard-Metal-only, CPU-only, Metal-or-CPU, and no-valid-route via the typed family policy from express. Negative cases are host/environment mismatch (or a consumer-chosen adapter absent for the selected family), incompatible-profile, and no-valid-route — not registry-era ~~missing-adapter~~ outcomes, which [`select-executable-variants-across-registered-backend-families`](select-executable-variants-across-registered-backend-families.md) records do not exist under the accepted no-registry model. ~~standard-Metal-only, custom-Metal-preferred, CPU-only, Metal-or-CPU, missing-adapter, incompatible-profile, and no-valid-route policies~~ **Imprecise:** custom-Metal-preferred is the within-Metal physical-provider axis, not a family-policy tier; missing-adapter is retired registry vocabulary.
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
