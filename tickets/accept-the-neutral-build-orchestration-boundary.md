---
id: accept-the-neutral-build-orchestration-boundary
title: Accept or revise the neutral build-orchestration boundary
status: awaiting-decision
priority: p1
dependencies: []
related: [promote-the-build-time-cache-and-correspondence-seam, produce-a-custom-backend-payload-through-the-build-orchestrator, carry-one-payload-per-artifact-family-in-one-envelope, audit-backend-authoring-against-all-thirteen-responsibilities, accept-the-public-backend-provider-composition-boundary]
scopes: [contracts/foundation, contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary, backend-providers]
---
## User-visible outcome

Tom accepts, revises, or refuses the exact public shape of `tiler-build`'s backend-neutral orchestration seam — the one surface an independent backend author must call to package an artifact — so that the seam an external backend depends on stops resting on a coordinator's provisional acceptance under names it has since outgrown.

## Why this node exists

**Fact — ADR 0090 routes this surface to Tom by name, and it landed without reaching him.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md):19 records that acceptance of that record "is of the *model*", and that "every concrete public surface named here — the provider registry and its installation method, the offered-versus-selected disclosure accessors, the promoted `assemble_artifact` boundary — still comes to Tom at implementation time under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md)". Item 11 repeats it: "The exposed function and its parameter types are a public boundary and therefore Tom's." The same paragraph records the promotion as landed on 2026-08-01. No acceptance ticket was filed for it, and none exists: `ls tickets/ | grep -i '^accept'` returns twenty-two nodes, none naming this seam.

**Fact — the only acceptance on record is a coordinator's, explicitly provisional.** [`promote-the-build-time-cache-and-correspondence-seam`](promote-the-build-time-cache-and-correspondence-seam.md):79 reads: "**Provisional boundary acceptance (2026-08-01, overnight mode).** The coordinator provisionally accepted the promoted seam … Recorded for Tom's morning review." Tom's live review on 2026-08-05 landed five acceptances at `25b636c6` — ADRs 0098 and 0099, the stage-coverage boundary, the debug-retention and `StageOutputs` surface, and the delivered-realization surface — and this seam was not among them.

**Fact — the provisionally accepted names are not the current names.** That ticket accepted `accept_or_publish_single_payload_artifact`, `SinglePayloadCacheError<M, C, A>`, and `SinglePayloadProtocolError<M>`. At `51e9374a` the exports are `accept_or_publish_delivered_payload_artifact`, `DeliveredPayloadCacheError<M, C, A>`, and `DeliveredPayloadProtocolError<M>`, and `CompiledPayloads` has been added since. So even the provisional acceptance does not describe the surface in the tree, and re-deriving what is actually being accepted is part of this node's work rather than a formality.

## The exact surface awaiting a decision

All at `crates/tiler-build/src/lib.rs:80-84`, from the private modules `payload_cache` and `plan_artifact`:

- `assemble_plan_artifact` (`plan_artifact.rs:152`), with its two closure parameters and the derived/delegated split its module documentation states.
- `BackendEntryDeclaration` (`plan_artifact.rs:59`), a public-field record of three statements — binding transports, the zero-work dispatch policy, and launch preconditions.
- `PlanArtifactError` (`plan_artifact.rs:77`).
- `accept_or_publish_delivered_payload_artifact` (`payload_cache.rs:520`), six arguments and three type parameters.
- `DeclaredPayload<'facts>` (`payload_cache.rs:137`), six public fields.
- `CompiledPayloads` (`payload_cache.rs:174`), two public fields, plus `From<Vec<PayloadContent>>`.
- `AcceptedArtifact` (`payload_cache.rs:205`) with `resolution`, `cache_subject`, `decoded`, `into_resolution`.
- `DeliveredPayloadCacheError<M, C, A>` (`payload_cache.rs:420`) and `DeliveredPayloadProtocolError<M>` (`payload_cache.rs:257`).

## What the packet must carry

- The three-error-parameter separation and the Metal one-to-one `From` mapping, which the provisional acceptance named as the design's load-bearing halves — a neutral record supporting only equality would collapse eleven named Apple facts into one undifferentiated refusal.
- The bounded case the seam does not orchestrate, stated in `lib.rs:43-47`: one payload per delivery position shared by every executable entry, with an artifact whose entries are realized by different objects at one position expressible in the artifact model and deliberately not orchestrated here.
- That `BindingKind` (`crates/tiler-artifact/src/program/model.rs:492`) has exactly one variant, so the transport category is a real parameter over a singleton domain today — the seam delegates a choice that currently has one answer, which is a reservation rather than an exercised degree of freedom.
- The conformance evidence, because it is what distinguishes this from an unexercised sketch: `crates/tiler-build/tests/custom_backend` is a non-Metal producer sharing no code with the Metal path, and `crates/tiler-build/Cargo.toml` declares no `[dev-dependencies]`, so it reaches the seam through the crate's public API and ordinary dependency closure alone.

## Closes when

Tom accepts, revises, or refuses each named item; the acceptance sentence names who accepted, the date, and the venue; ADR 0090's status paragraph and item 11 record the outcome; and any revision is released to its own implementation ticket rather than landed under this node.
