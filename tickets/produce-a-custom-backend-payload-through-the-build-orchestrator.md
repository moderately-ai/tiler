---
id: produce-a-custom-backend-payload-through-the-build-orchestrator
title: Produce a custom backend payload through the build orchestrator
status: in-progress
priority: p1
dependencies: [accept-the-public-backend-provider-composition-boundary]
related: [drive-the-build-orchestrator-from-a-checked-compiler-plan, assemble-prepared-metal-artifacts-in-tiler-build]
scopes: [implementation/build, implementation/compiler, implementation/artifact, contracts/artifacts, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, build, artifacts]
---
## User-visible outcome

A statically linked custom backend producer can consume verified compiler output through the build orchestrator and publish one canonical backend payload without forging derived identity or coupling the compiler core to that backend.

## Implementation keys

- Define the smallest accepted emitter/artifact-producer facade; do not expose `tiler-build` internals as the generic model.
- Feed only verified structured KIR/program products and the accepted target/profile request into the producer.
- Separate pure emission from external AOT/tool invocation and artifact assembly even when one backend implements both.
- Derive backend, representation, entry mapping, compilation subject, payload digest, target obligations, and cache subject through their owning checked builders.
- Preserve complete source/toolchain/flag/provenance identity and reject mismatched profile or ABI before external compilation.
- Support a producer that is not Metal and a partial custom Metal provider that reuses standard Metal emission where the accepted composition permits it.
- Prove malformed entry mappings, unstable identity, duplicate producers, forged payload facts, and cache subject disagreement fail.
- Present the exact public trait/type/call-site boundary to Tom.

## Closes when

One external producer creates a decoded, self-validating payload through the ordinary build path, byte and identity determinism are demonstrated, mutation tests move every affected identity, targeted checks pass, and the standard Metal path remains behaviorally unchanged.

## Graph maintenance

- Feed the payload into runtime-adapter and cross-process join tickets.
- Keep dynamic plugin loading and provider discovery out of scope.
- If a new crate is required, split crate admission into its own Tom-reviewed ticket before implementation.

## Outcome

**Landed.** `tiler-build`'s private `assemble_artifact` is promoted to the public `tiler_build::assemble_plan_artifact`, per accepted [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 11's exact recommendation — a promoted closure seam, no backend trait, no new crate. `BindingKind`, the zero-work dispatch policy, and the launch preconditions moved from the orchestrator's assumption into a backend-supplied `BackendEntryDeclaration`; the private `PlanArtifactError` is now the facade's public typed refusal. `crates/tiler-build/src/plan_artifact.rs` is the seam; the Metal path is one caller and shares no code with the other.

**The D10 refutation condition did not fire.** The condition is *a second backend needing to vary something `assemble_artifact` derives rather than something it delegates*. The non-Metal producer in `crates/tiler-build/tests/custom_backend` — its own governed profile key, backend family, executable representation, in-process translator, non-identity transport map, launch precondition, and zero deferred predicates — wanted nothing the facade derives. Four things it might have wanted are derived and were not missed: the target-profile reference and exact descriptor digest, the feasibility rule set, the selected providers, and each entry's `BackendEntryKey`. The split is where item 11 said it was.

**One structural improvement over the sketch.** Entries are declared by calling the backend once per stage rather than by returning a `Vec`, so a producer cannot declare more or fewer entries than the plan has stages; the cardinality is unrepresentable rather than checked.

**Metal path behaviourally unchanged, proven by pinned goldens.** `the_standard_metal_path_publishes_its_recorded_identities` records the artifact canonical identity and the composed cache subject captured at `63f9259` *before* the rewrite; both are byte-identical after it. Perturbing the facade's derived entry key or descriptor moves them, so the goldens can say no.

**Not done, deliberately, and named.** Cache-subject composition, miss-only external compilation, and payload correspondence validation in `metal_cache.rs` remain spelled for one prepared Metal compilation. Item 11 promotes the assembly seam only, and generalizing the cache protocol would expose `tiler-build` internals as the generic model — the first implementation key's explicit warning. The non-Metal producer composes its own subject through `tiler_cache::expansion::ComposedSubject::compose` (the owning checked builder) and re-checks identity after resolution; [`promote-the-build-time-cache-and-correspondence-seam`](promote-the-build-time-cache-and-correspondence-seam.md) owns closing that.

**Public boundary for Tom.** `assemble_plan_artifact`, `BackendEntryDeclaration`, and `PlanArtifactError` go to him under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md), with ADR 0090 item 11's acceptance as the umbrella for the promotion itself and the entry-declaration record as the one genuinely new shape.
