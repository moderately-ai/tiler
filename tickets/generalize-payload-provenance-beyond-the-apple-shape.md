---
id: generalize-payload-provenance-beyond-the-apple-shape
title: Generalize payload provenance beyond the Apple shape
status: in-progress
priority: p2
dependencies: []
related: [route-or-refuse-the-device-translation-execution-policy, prototype-a-bounded-scalar-cpu-backend-vertical, state-the-backend-payload-validation-obligation-normatively]
scopes: [implementation/artifact, contracts/artifacts]
shared_scopes: [project/tickets]
paths: []
tags: [artifacts, backend-providers, provenance, consumer-neutrality]
claimed_from: todo
assignee: coordinator
lease_expires_at: 1785612420
---
## User-visible outcome

A non-Apple backend can state its payload's provenance in terms that mean something for it, instead of filling Apple-shaped fields with values that have no referent on its target.

## Why this exists

**Fact — the provenance record is Apple-shaped.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md):125, under item 14 at `:123`, names it among four vocabulary gaps each measured by the CPU vertical: "`PayloadProvenance` requires Apple-shaped fields with no CPU meaning". The type is at `crates/tiler-artifact/src/program/codec/payload.rs:144`, carried on the payload at `:209`, encoded at `:304`, and decoded at `:474`/`:507`; its Apple-specific companions include `PayloadSdkIdentity`, and every production construction site is a Metal one (`crates/tiler-build/src/metal_payload.rs:339`, `crates/tiler-build/src/metal_assembly.rs:350`).

**Fact — the CPU target is now live, not hypothetical.** [`prototype-a-bounded-scalar-cpu-backend-vertical`](prototype-a-bounded-scalar-cpu-backend-vertical.md) is `done` and carried a second backend end to end, and the CPU vector-lane tier's ADR 0093 is accepted with its three implementation tickets filed. The seam is being pressed by real work rather than reserved against a future one.

**Fact — the condition the CPU vertical set for filing has passed.** `prototype-a-bounded-scalar-cpu-backend-vertical.md:62` withheld a ticket per seam because "filing a ticket per seam before that contract exists would pre-empt its design". The contract exists and ADR 0090 was accepted on 2026-07-31 (`:19`).

**Inference — this is the consumer-neutrality invariant, applied to a field.** AGENTS.md requires the compiler core to stay independent of Metal runtime objects and other consumer-specific types, and requires unsupported cases to reject explicitly rather than approximate. A backend forced to mint an SDK identity it has no notion of is approximating — and an approximated field that enters durable identity is worse than a missing one, because it makes two unlike artifacts comparable.

## Boundaries

- Provenance enters artifact identity. Any reshaping states exactly what moves, what stays put, and whether the identity domain steps; an appended-tag widening that moves no previously encodable payload's bytes is the shape to aim for.
- Generalizing must not weaken what Apple provenance currently proves. A field a Metal payload must carry stays required for a Metal payload; the generalization is about which backend owes which field, not about making all of them optional.
- The normative statement of the payload-validation obligation is [`state-the-backend-payload-validation-obligation-normatively`](state-the-backend-payload-validation-obligation-normatively.md)'s; this ticket shapes the provenance record, not the validation schedule.

## Closes when

An out-of-crate non-Metal backend constructs a payload whose provenance is complete and meaningful for its target, with no field defaulted to an Apple-shaped placeholder; a payload omitting a field its own backend owes is refused by name, observed failing; the identity consequences are stated exactly; and ADR 0090:125's sentence naming this seam is corrected.
