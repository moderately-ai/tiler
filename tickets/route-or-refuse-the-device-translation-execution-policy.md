---
id: route-or-refuse-the-device-translation-execution-policy
title: Route or retire the device-translation execution policy
status: todo
priority: p2
dependencies: []
related: [generalize-payload-provenance-beyond-the-apple-shape, specify-the-consumer-neutral-backend-provider-composition-contract]
scopes: [implementation/artifact, implementation/runtime]
shared_scopes: [project/tickets]
paths: []
tags: [runtime, artifacts, backend-providers, routing]
---
## User-visible outcome

`ArtifactExecutionPolicy` stops being a two-valued vocabulary one of whose values cannot route: a representation needing device-side translation either reaches a device, or the vocabulary stops claiming it is expressible.

## Why this exists

**Fact — one of two values is unroutable, deliberately.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md):125, under its item-14 heading at `:123` ("Every other unsupported case rejects explicitly"), names it: "`ArtifactExecutionPolicy` is a two-valued GPU dichotomy of which one value is unroutable, since [the loader] returns `LoadRejection::UndeliverableExecutionPolicy` for `RequiresDeviceTranslation` deliberately rather than by wildcard, so a representation needing device-side translation cannot route at all today." The refusal is at `crates/tiler-runtime/src/load.rs:549`, matching `policy @ ArtifactExecutionPolicy::RequiresDeviceTranslation`. The variant itself is `crates/tiler-artifact/src/program/model.rs:420`, tag `0x02` at `:427`.

**Fact — the condition set for filing this has passed.** [`prototype-a-bounded-scalar-cpu-backend-vertical`](prototype-a-bounded-scalar-cpu-backend-vertical.md):62 recorded why no ticket was filed per seam at the time: "the four vocabulary seams above are inputs to the composition contract rather than defects, and filing a ticket per seam before that contract exists would pre-empt its design." That contract now exists — [`specify-the-consumer-neutral-backend-provider-composition-contract`](specify-the-consumer-neutral-backend-provider-composition-contract.md) and [`draft-the-backend-provider-composition-adr`](draft-the-backend-provider-composition-adr.md) are `done`, and ADR 0090 was accepted on 2026-07-31 (`:19`).

**Inference — an unroutable enum value is a claim the vocabulary cannot cash.** AGENTS.md requires unsupported cases to reject explicitly rather than silently approximate, and this one does — the refusal is by named variant rather than by wildcard, which is correct. What is not correct is leaving a *vocabulary* asserting that a device-translated representation is expressible while nothing can deliver one. Either the route exists or the value does not.

## Run the elimination and state which candidate survived

- **Route it.** A representation requiring device-side translation reaches a device through the adapter seam, with the translation's authority, its identity contribution, and its failure staging all stated. What this must answer is which backend performs the translation and when — before the preflight is held, or after, which ADR 0051's one-way commit constrains sharply.
- **Retire it.** The variant is removed and the policy stops being a dichotomy. This moves an artifact-identity tag, so it is not free: state what happens to the `0x02` encoding, and whether the domain steps.

Whichever survives, the elimination is written out so a reader can refute it. Do not close this by observing that the refusal is correct today — the refusal being correct is the premise, not the outcome.

## Closes when

One candidate lands with its derivation stated; the refusal at `crates/tiler-runtime/src/load.rs:549` either stops being reachable-by-construction or is replaced by the variant's removal; ADR 0090:125's sentence naming this seam is corrected; and identity consequences are stated exactly, with any encoding change shown to move no previously encodable artifact's bytes or to step the domain deliberately.
