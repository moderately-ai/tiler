---
id: accept-the-loader-variant-eligibility-vocabulary
title: Accept or revise the loader variant-eligibility vocabulary
status: awaiting-decision
priority: p2
dependencies: [select-executable-variants-across-registered-backend-families]
related: [expose-explicit-backend-provider-and-selection-policy-composition, decide-whether-a-loading-host-may-state-several-backend-families, accept-the-neutral-build-orchestration-boundary]
scopes: [contracts/decisions]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [runtime, public-boundary, decision, needs-tom]
---
## User-visible outcome

The `tiler_runtime::load` variant-eligibility vocabulary rests on Tom's acceptance rather than on a coordinator's provisional overnight one, so the runtime half of the backend-composition surface stops being the only one of the three that never reached him.

## Why this exists

**Fact — the surface was provisionally accepted by the coordinator and recorded for Tom, and no node carries that review.** [`select-executable-variants-across-registered-backend-families`](select-executable-variants-across-registered-backend-families.md) closed `done` with a section headed "Provisional boundary acceptance (2026-08-01, overnight mode)" which states the coordinator accepted the loader vocabulary change and that it was "Recorded for Tom's morning review". No `accept-*` ticket names it. The three that come closest — [`accept-the-public-route-requirement-answer-boundary`](accept-the-public-route-requirement-answer-boundary.md), [`accept-the-route-facts-dtype-dispatch-field`](accept-the-route-facts-dtype-dispatch-field.md), and [`accept-the-route-resource-requirement-spelling`](accept-the-route-resource-requirement-spelling.md) — are all about route *requirements* and none mentions variant eligibility.

**Fact — the precedent says a provisional acceptance is superseded rather than left standing.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md):19 records that the build orchestration surface "was accepted by Tom on 2026-08-05 at the live decision review under [`accept-the-neutral-build-orchestration-boundary`](accept-the-neutral-build-orchestration-boundary.md), superseding the provisional overnight acceptance it had rested on". The compiler surface has its node in [`accept-the-installed-physical-provider-public-surface`](accept-the-installed-physical-provider-public-surface.md). The runtime surface has neither.

## The surface, as it stands at `750b29e0`

Read from `crates/tiler-runtime/src/load.rs` at this base rather than copied from the originating ticket's summary, because the two differ: the fourth `VariantIneligibility` class arrived later, in `63109caa`, and is therefore in the accepted set even though that ticket's Outcome names three.

**Added to `tiler_runtime::load`.** `VariantIneligibility` (`#[non_exhaustive]`; `Clone + Debug + Eq + Hash + PartialEq + Display`) with `AssessedProfile { classification }`, `UnsupportedRepresentation { entry, declared_backend, declared_representation, host_backend, host_representation }`, `PayloadProfile { entry, classification }`, and `UndispatchableDType { entry, arithmetic, resolution, host_profile }`; `FilteredVariant` with public fields `variant` and `reason`; `LoadRejection::NoEligibleVariant { packaged, filtered }`.

**Changed.** `LoadRejection::NoApplicableVariant` gained `filtered`.

**Removed.** `LoadRejection::UnexecutablePayload`, `LoadRejection::IncompatibleTarget`, and `TargetDeclaration` with its `Display`. This is the part most worth a deliberate answer: it is a removal from a published surface, justified under pre-alpha superseded-path discipline on the ground that the finer vocabulary subsumes them with a filtered-versus-failed distinction the old classes could not carry.

**Unchanged, and stated so the accepted set has an excluded half.** `ExecutionEnvironment`, `TargetCompatibility`, and every other loader type; no signature on `preflight`, `prepare`, `route_with_adapter`, or the `RuntimeAdapter` trait moved.

## The questions that are genuinely Tom's

1. **Is removing the three superseded classes right**, or should they have been retained as deprecated aliases? Pre-alpha discipline says remove; the counterpoint is that `UnexecutablePayload` is cited by name in ADR 0090 item 8's own corrected text and in a spike outside the gate.
2. **Are `VariantIneligibility`'s four classes the right cut?** Each names a different repair. The counterpoint is `UndispatchableDType`, whose necessity rests on both consumer paths restating the producer's declaration rather than the host's own — a tautology the loader documents but does not fix.
3. **Should `FilteredVariant`'s fields be public**, or accessors as the rest of this crate uses?

## Recommendation

Accept removal of the three superseded broad classes and accept the four current ineligibility classes: they preserve the filtered-versus-failed distinction and give each current refusal a different repair. Keep `FilteredVariant` immutable but replace its public fields with accessors for consistency with the rest of the loader surface. **Strongest counterpoint:** the two fields are plain immutable evidence and public access is simpler; changing them now adds ceremony without changing invariants. [`declare-host-dtype-dispatchability-at-the-consumer-boundary`](declare-host-dtype-dispatchability-at-the-consumer-boundary.md) already records why the current frontend and Candle rows remain producer-declared and what a host-earned row would require, so accept `UndispatchableDType` as the fail-closed class for that explicitly bounded state rather than filing a duplicate gap ticket.

## Closes when

Tom accepts or revises each, the provisional-acceptance paragraph in the originating ticket is annotated with what superseded it, and the acceptance provenance — who, date, venue, relay source — is recorded.

## Graph maintenance

- Only Tom closes this. An agent may not convert a coordinator's provisional acceptance into a durable one by restating it.
- A revision that changes a class is an implementation change; file it rather than editing the landed record to match.
