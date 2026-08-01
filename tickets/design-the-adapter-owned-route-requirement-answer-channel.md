---
id: design-the-adapter-owned-route-requirement-answer-channel
title: Design the adapter-owned route-requirement answer channel
status: in-progress
priority: p1
dependencies: []
related: [dispatch-a-tiler-region-on-metal-hardware, route-an-embedded-artifact-through-a-consumer-storage-seam, realize-parallel-reduction-strategies-on-metal]
scopes: [research/runtime, research/extensions]
shared_scopes: [project/tickets]
paths: []
tags: [research, design, runtime, backend-providers, public-boundary]
claimed_from: todo
assignee: worker-answer-channel
lease_expires_at: 1785594086
---
## User-visible outcome

A dispatching consumer can answer a backend-typed live-device route requirement — `tiler.metal.route-requirement.minimum-gpu-family` is the concrete case, one `MTLDevice::supportsFamily` call away — through a designed channel, instead of every such row being permanently `Unrecognized` on the facade path.

## The decision Tom made, and what remains

**Fact — reviewed 2026-08-01.** Tom reviewed three candidates and chose the design-ticket route with a stated lean toward the adapter-owned channel, with fail-closed as the explicit interim:

- **(a) Re-export the `tiler-metal` vocabulary through the facade — eliminated.** No facade-reachable signature names `MetalGpuFamily` (the consumer must *produce* one, not read one, so the `tiler::runtime` re-export precedent does not apply); it would put a backend crate in every consumer's closure including fallback-only ones; and a second backend would add a second crate.
- **(c) Fail-closed forever — rejected as a terminus, accepted as the interim.** `spikes/runtime/inline-dispatch` answers `Unrecognized` today and that stays correct while this design runs. As an end state it leaves the compiler minting requirements nothing on the primary consumer path can answer.
- **(b2) Adapter-owned answers — the lean this ticket derives or refutes.** A *dispatching* consumer is already backend-specific (the spike links `metal` itself); the facade rule "a consumer names `tiler` alone" is a property of the fallback path. The shape to derive: the applicability vocabulary becomes a deliberately public, versioned boundary of `tiler-metal` that the consumer's **adapter** — the device authority that observed the device — may name; the runtime validates the constructed answer against the carried requirement without the neutral layer naming the backend. The dependency arrow stays consumer→backend, never core→backend.
- **(b1) Neutralize the vocabulary into the runtime/artifact layer** — carried as the alternative b2 must eliminate on the record rather than by assertion. Its stated hazard: the value set is irreducibly a backend fact, so a neutral carrier is either a disguised backend registry or opaque bytes — an unvalidated second authority.

## Questions the design must answer, each with its elimination stated

- Exactly which items of `tiler_metal::applicability` go public, under what versioned identity, and what the compatibility contract of that boundary is — a backend vocabulary a consumer names is a surface that can no longer be reshaped freely.
- How the neutral runtime validates a constructed answer against the carried requirement without depending on the backend: what travels typed, what travels canonical-bytes-with-backend-validation, and where the validation authority lives.
- How ADR 0086's eligibility gate composes with an answered row: answering a GPU-family requirement is a device *capability* fact, not translation attestation — state precisely which routing conclusions an answer does and does not license, so a satisfied family row is never read as host-earned translation eligibility.
- What a second backend (the CPU family is the live candidate under the current target-device priorities) does with the same channel — the design generalizes or says why it deliberately does not.
- What the fallback-only consumer's contract remains: naming `tiler` alone must stay sufficient for every non-dispatching use.

## Closes when

The channel is designed with the b1/b2 elimination written where a reader can refute it, the exact public boundary items are enumerated and taken to Tom under ADR 0075 rather than self-accepted, the interim fail-closed behaviour is restated in the spike and route documentation as deliberate, and the outcome is an accepted design, a recorded deferral with trigger, or a bounded experiment.
