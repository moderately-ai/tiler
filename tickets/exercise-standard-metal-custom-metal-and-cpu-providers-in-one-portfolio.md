---
id: exercise-standard-metal-custom-metal-and-cpu-providers-in-one-portfolio
title: Exercise standard Metal, custom Metal, and CPU providers in one portfolio
status: done
priority: p1
dependencies: [expose-explicit-backend-provider-and-selection-policy-composition, join-build-time-producers-to-runtime-adapters-through-artifact-identity]
related: [prototype-inline-aot-integration-proof, prototype-metal-runtime-proof, decide-whether-a-loading-host-may-state-several-backend-families, select-executable-variants-across-registered-backend-families, publish-the-backend-provider-conformance-suite]
scopes: [research/runtime, research/extensions, research/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, integration, metal, cpu]
---
## User-visible outcome

One retained end-to-end proof composes standard Metal, a forkless custom Metal specialization, and a bounded CPU backend, packages their valid alternatives, explicitly routes separate Metal and CPU attempts, and matches the independent reference result.

## Implementation keys

**Corrected 2026-08-10 against accepted [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 4 (no `BackendProvider` / no provider bundle) and the [`select-executable-variants-across-registered-backend-families`](select-executable-variants-across-registered-backend-families.md) Outcome (no-registry routing).** Custom Metal is a physical-provider row claim on the Metal family, not a third backend family: the portfolio story is two physical authorities under Metal plus one CPU family (and their adapters/payloads), under one semantic program. Family-policy vocabulary is owned by [`express-the-typed-backend-family-selection-policy`](express-the-typed-backend-family-selection-policy.md) (blocked on [`decide-whether-a-loading-host-may-state-several-backend-families`](decide-whether-a-loading-host-may-state-several-backend-families.md)); this ticket exercises those seams end to end once they land.

**Corrected 2026-08-11 by Tom's explicit-backend decision.** The family-policy sentence above is retained as dated history and no longer describes live work. [`express-the-typed-backend-family-selection-policy`](express-the-typed-backend-family-selection-policy.md) is closed `wontdo`: each attempt explicitly states one backend environment, and neither the loader nor a consumer facade silently falls back across families. This proof exercises the same packaged portfolio through separate Metal and CPU attempts and proves that presenting either family under the other's environment refuses during preflight.

- Use one semantic program that exercises the custom Metal candidate while retaining a valid standard Metal and CPU route.
- Drive the accepted per-responsibility composition path end to end — compile: `CompileRequest::with_physical_providers` / offered+selected physical provenance; build: `assemble_plan_artifact` / payload publish; runtime: one explicitly chosen host `ExecutionEnvironment` + `route_with_adapter` per attempt — and record exact physical/provider selection provenance. ~~Compile through the public provider composition facade~~ **False name relative to ADR 0090 item 4:** there is no `BackendProvider`, no provider bundle, and no single facade type; composition is the per-row seams above.
- Produce backend payloads without duplicating semantic meaning or runtime adapter logic.
- Package a complete portfolio whose members vary backend, representation, payload profile, and compilation subjects independently while sharing one assessed variant-level `TargetProfileRef` (and one numerical contract) pinned by artifact `check_subject`. ~~independent backend, representation, target, and compilation identities~~ **Imprecise on target:** variant-level target is one subject for the portfolio; independence holds for payload-side profile / backend / representation / compilation, not for a second assessed variant target.
- Run CPU on every host; run Metal legs only on eligible measured hosts and report explicit unavailability rather than silently passing.
- Exercise two axes without turning them into one policy list: (i) **within Metal** — governed vs installed custom physical-provider selection among plan alternatives; (ii) **across families** — one explicit Metal attempt and one explicit CPU attempt against the same portfolio. Cross-family negative controls present each family's **own** assembled artifact under the other environment and require preflight refusal before work. Against the combined portfolio the loader selects the matching family; that is eligibility, not fallback. There is no Metal-or-CPU fallback policy and no registry-era ~~missing-adapter~~ outcome.
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

## Fact audit at `61246804`

Re-read 2026-08-12 at this ticket's dispatch base. Every verdict below names the file that was opened.

| Claim | Verdict | Evidence |
| --- | --- | --- |
| No `BackendProvider` / no provider bundle | **Verified** | [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 4: `There is no BackendProvider, no provider bundle`. `grep` for `struct BackendProvider` / `trait BackendProvider` over `*.rs` returns nothing. |
| Custom Metal is a physical-provider row on the Metal family, not a third family | **Verified** | Same item 4; the forkless provider reuses `tiler_metal::emit` and never mints a third `BackendKey`. |
| [`express-the-typed-backend-family-selection-policy`](express-the-typed-backend-family-selection-policy.md) is closed `wontdo` | **Verified** | That ticket's frontmatter: `status: closed`, `closed_reason: wontdo`. |
| Each attempt states one `ExecutionEnvironment` | **Verified** | `crates/tiler-runtime/src/load/host.rs`: `One explicit backend choice per routing attempt`; the struct carries one `BackendKey` and one `RepresentationKey`. |
| Neither loader nor facade silently falls back across families | **Verified** | Same module: `It never retries another family.` |
| `CompileRequest::with_physical_providers` exists | **Verified** | `crates/tiler-compiler/src/session.rs` `pub fn with_physical_providers`. |
| Offered and selected physical provenance exist | **Verified** | `Compilation::offered_physical_providers`, `PlanAlternative::selected_physical_providers`. |
| `assemble_plan_artifact` is the build seam | **Verified** | `crates/tiler-build/src/plan_artifact.rs` `pub fn assemble_plan_artifact`. |
| Payload publish exists | **Verified** | `accept_or_publish_delivered_payload_artifact` and `accept_or_publish_metal_plan`. |
| Runtime join is one `ExecutionEnvironment` + `route_with_adapter` | **Verified** | `crates/tiler-runtime/src/adapter.rs` `pub fn route_with_adapter`. |
| There is no single composition facade type | **Verified** | ADR 0090 item 4; the parent ticket's corrected keys name the per-row seams. |
| Variant-level target is one subject per artifact | **Verified** | `ArtifactProgramBuilder::check_subject` returns `ArtifactBuildError::TargetProfileMismatch` when `subject.profile != spec.target_profile`. |
| Independence holds for payload profile / backend / representation / compilation, not for a second assessed variant target | **Verified** | Same `check_subject`; `variant_eligibility` classifies payload compatibility separately as `PayloadProfile`. |
| Both dependencies are `done` | **Verified** | [`expose-explicit-backend-provider-and-selection-policy-composition`](expose-explicit-backend-provider-and-selection-policy-composition.md) and [`join-build-time-producers-to-runtime-adapters-through-artifact-identity`](join-build-time-producers-to-runtime-adapters-through-artifact-identity.md) frontmatter `status: done`. |
| `make full` does not reach `spikes/` | **Verified** | `Makefile`: `Spikes deliberately have no target.` |

**Corrected 2026-08-12 from the spike run.** Two live sentences were imprecise:

- `STRICT_F32` against `BoundMetalCompileDeclaration::first_macos_apple9` is `NoFeasiblePlan`. The Apple declaration assesses `FLUSH_SUBNORMALS_TO_ZERO_F32`, which is what `prototypes/serial-sum-compile` already compiles under. The spike uses that contract.
- "Presenting either family under the other's environment" against the **combined** portfolio does not refuse: eligibility selects the matching member. The refusal is observed by presenting each family's **own** `assemble_plan_artifact` / `accept_or_publish_metal_plan` artifact under the other `ExecutionEnvironment`. The implementation key above is repaired to say that.

## Worker report

Spike retained at [`spikes/runtime/backend-provider-portfolio`](../spikes/runtime/backend-provider-portfolio/README.md). Command:

```sh
cd spikes/runtime/backend-provider-portfolio
CARGO_TARGET_DIR=./target cargo run -- results/2026-08-12-macos-arm64.json
```

**Measurement** at base `61246804` on this host: offered physical environment with the custom provider is `tiler::prototype-serial-sum-physical@1,acme::simdgroup-pointwise-metal@4`; two retained alternatives name those providers separately; removing the custom provider leaves only the governed identity; mixed-target merge is `TargetProfileMismatch`; Metal-only under CPU and CPU-only under Metal are `runtime.no-eligible-variant` / `UnsupportedRepresentation`; CPU and Metal each produce twelve bit patterns equal to `tiler-reference`, including the NaN operand canonicalized from `0x7fc01234` to `0x7fc00000`. Fixture: [`spikes/runtime/backend-provider-portfolio/results/2026-08-12-macos-arm64.json`](../spikes/runtime/backend-provider-portfolio/results/2026-08-12-macos-arm64.json).

No `crates/` path was edited. No production defect ticket was required: `STRICT_F32` infeasibility is the Apple declaration working, and deferred predicates on a Metal-assessed CPU member are the Apple profile's prepared-entry query working. Cost-comparability of the two Metal providers remains the open question ADR 0090 already recorded.
