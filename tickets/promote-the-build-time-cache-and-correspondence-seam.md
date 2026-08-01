---
id: promote-the-build-time-cache-and-correspondence-seam
title: Promote the build-time cache and correspondence seam
status: todo
priority: p2
dependencies: []
related: [produce-a-custom-backend-payload-through-the-build-orchestrator]
scopes: [implementation/build]
shared_scopes: [project/tickets]
paths: []
tags: [backend-providers, pluggability, implementation, build, cache]
---
## User-visible outcome

A backend that is not Metal reaches cache-subject composition, miss-only external compilation, and payload correspondence validation through `tiler-build` rather than reimplementing them, without the Metal cache protocol becoming the generic model.

## Why this is separate

[ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md) item 11 promotes the *assembly* seam and names nothing else; `produce-a-custom-backend-payload-through-the-build-orchestrator` landed exactly that. What stayed Metal-shaped is `crates/tiler-build/src/metal_cache.rs`: `accept_or_publish_single_payload_metal_artifact` takes a `PreparedMetalPayload`, validates the descriptor against the hardcoded `tiler.metal`/`metallib`/`NativeImage` constants, and runs `validate_metal_payload_metadata` — a fact-level Apple correspondence check — inside the miss closure and again after resolution.

The reason it was not promoted with the assembly seam is a finding rather than an omission: the cache orchestration's structural obligations (subject composition, identity agreement before publication, re-validation of every result) are **interleaved** with payload-specific validation at three points, so a single closure parameter does not factor them the way `assemble_artifact`'s did. Promoting it therefore needs a design decision, not a move.

## Implementation keys

- Decide whether the neutral shape is a structural facade with a post-decode hook, a declaration record plus one compile closure, or a split into two functions; state the elimination.
- Keep the artifact layer's derivation intact: the payload digest is derived from canonical metadata bytes, the composed subject from `tiler_cache::expansion::ComposedSubject::compose`.
- Preserve the Metal path's exact refusal kinds and their order. `MetalPayloadFact`-level diagnostics are a refinement over the descriptor-digest comparison that already subsumes them; losing them is a real regression in explainability even though it changes no accept/reject decision.
- Keep the standard Metal artifact identity and composed cache subject byte-identical; `the_standard_metal_path_publishes_its_recorded_identities` in `crates/tiler-build/src/metal_plan.rs` is the pinned evidence.
- Extend the non-Metal producer in `crates/tiler-build/tests/custom_backend` to drive the promoted path instead of `backend::accept_or_publish`, and delete that helper when it is superseded.

## Closes when

The non-Metal producer publishes and re-accepts through the promoted seam, the Metal goldens have not moved, every refusal has been watched failing, and the public boundary is in a packet for Tom.

## Graph maintenance

- Do not introduce a new crate; if one seems required, stop and report.
- Keep dynamic plugin loading and provider discovery out of scope.
