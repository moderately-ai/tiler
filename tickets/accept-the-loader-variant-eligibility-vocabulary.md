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

The `tiler_runtime::load` variant-eligibility vocabulary rests on Tom's acceptance rather than on a coordinator's provisional overnight one, so the runtime half of the backend-composition surface stops resting solely on that provisional record.

**Correction — 2026-08-10.** Earlier wording said runtime was "the only one of the three that never reached him." Among the three composition-adjacent public surfaces, build orchestration is Tom-accepted (2026-08-05 under [`accept-the-neutral-build-orchestration-boundary`](accept-the-neutral-build-orchestration-boundary.md)); installed physical-provider and this loader vocabulary both still await Tom. Runtime is not uniquely node-less: this ticket is its accept node.

## Why this exists

**Fact — the surface was provisionally accepted by the coordinator and recorded for Tom, and no Tom decision yet supersedes that provisional vocabulary.** [`select-executable-variants-across-registered-backend-families`](select-executable-variants-across-registered-backend-families.md) closed `done` with a section headed "Provisional boundary acceptance (2026-08-01, overnight mode)" which states the coordinator accepted the loader vocabulary change and that it was "Recorded for Tom's morning review". No *other* `accept-*` ticket names that provisional review; **this ticket is the carrier node** for Tom's durable answer. Absence of a superseding Tom decision is not absence of a ticket. The three near-miss accept nodes — [`accept-the-public-route-requirement-answer-boundary`](accept-the-public-route-requirement-answer-boundary.md), [`accept-the-route-facts-dtype-dispatch-field`](accept-the-route-facts-dtype-dispatch-field.md), and [`accept-the-route-resource-requirement-spelling`](accept-the-route-resource-requirement-spelling.md) — are all about route *requirements* and none mentions variant eligibility.

**Fact — the precedent says a provisional acceptance is superseded rather than left standing.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), anchor `superseding the provisional overnight acceptance`, records that the build orchestration surface was accepted by Tom on 2026-08-05 at the live decision review under [`accept-the-neutral-build-orchestration-boundary`](accept-the-neutral-build-orchestration-boundary.md). The compiler surface has its node in [`accept-the-installed-physical-provider-public-surface`](accept-the-installed-physical-provider-public-surface.md) (still `awaiting-decision`). This ticket is the corresponding accept node for the runtime loader eligibility vocabulary; Tom has not yet closed it for the original three-class provisional set plus the selection-refusal split and removals.

**Fact — a subset of the live vocabulary already has first-hand Tom acceptance.** [`validate-bf16-at-the-runtime-routing-boundary`](validate-bf16-at-the-runtime-routing-boundary.md), section "Routing surface — accepted", records that Tom accepted on 2026-08-06 (morning decision review, coordinator-witnessed): `ExecutionEnvironment.dtype_dispatch`, `DTypeDispatch`, `DTypeDispatchResolution` with silence refusing, `VariantIneligibility::UndispatchableDType`, and the deliberate non-spelling of `Deferred` at routing. That fourth ineligibility class is therefore not still resting solely on the 2026-08-01 provisional overnight acceptance. Open questions below do not re-litigate that class as if never reviewed.

## The surface, as it stands at `750b29e0`

Read from `crates/tiler-runtime/src/load.rs` at this base rather than copied from the originating ticket's summary, because the two differ: the fourth `VariantIneligibility` class arrived later under [`validate-bf16-at-the-runtime-routing-boundary`](validate-bf16-at-the-runtime-routing-boundary.md) and is therefore in the inventory even though the originating ticket's Outcome names three.

**Added to `tiler_runtime::load`.** `VariantIneligibility` (`#[non_exhaustive]`; `Clone + Debug + Eq + Hash + PartialEq + Display`) with `AssessedProfile { classification }`, `UnsupportedRepresentation { entry, declared_backend, declared_representation, host_backend, host_representation }`, `PayloadProfile { entry, classification }`, and `UndispatchableDType { entry, arithmetic, resolution, host_profile }`; `FilteredVariant` with public fields `variant` and `reason`; `LoadRejection::NoEligibleVariant { packaged, filtered }`.

**Changed.** `LoadRejection::NoApplicableVariant` gained `filtered`.

**Removed.** `LoadRejection::UnexecutablePayload`, `LoadRejection::IncompatibleTarget`, and `TargetDeclaration` with its `Display`. This is the part most worth a deliberate answer: it is a removal from a published surface, justified under pre-alpha superseded-path discipline on the ground that the finer vocabulary subsumes them with a filtered-versus-failed distinction the old classes could not carry.

**Unchanged, and stated so the accepted set has an excluded half.** `ExecutionEnvironment`, `TargetCompatibility`, and every other loader type; no signature on `preflight`, `prepare`, `route_with_adapter`, or the `RuntimeAdapter` trait moved.

## The questions that are genuinely Tom's

1. **Is removing the three superseded classes right**, or should they have been retained as deprecated aliases? Pre-alpha discipline says remove; the counterpoint is that `UnexecutablePayload` is cited by name in ADR 0090 item 8's own corrected text and in a spike outside the gate.
2. **Do the remaining three ineligibility classes (`AssessedProfile`, `UnsupportedRepresentation`, `PayloadProfile`), the removals, and the selection refusal split (`NoEligibleVariant` vs `NoApplicableVariant` with `filtered`) cohere with the already-accepted fourth class `UndispatchableDType`?** Each of the four names a different repair. The counterpoint on the fourth class is not whether to admit it (Tom already accepted it on 2026-08-06) but that its practical necessity still rests on both consumer paths restating the producer's declaration rather than the host's own — a tautology the loader documents and [`declare-host-dtype-dispatchability-at-the-consumer-boundary`](declare-host-dtype-dispatchability-at-the-consumer-boundary.md) already records, rather than a second open gap.
3. **Should `FilteredVariant`'s fields stay public**, or move to accessors as a style preference? Route result types in `load/route.rs` use private fields with accessors; host evidence records in the same public module tree (`ExecutionEnvironment`, and currently `FilteredVariant`) use public fields. There is no single crate-wide convention that forces either choice.

## Recommendation

Accept removal of the three superseded broad classes and accept the three original ineligibility classes plus the selection-refusal split as cohere with the already-accepted `UndispatchableDType`: they preserve the filtered-versus-failed distinction and give each current refusal a different repair. Leave the already-accepted `UndispatchableDType` in place; do not file a duplicate gap ticket — [`declare-host-dtype-dispatchability-at-the-consumer-boundary`](declare-host-dtype-dispatchability-at-the-consumer-boundary.md) already records why the current frontend and Candle rows remain producer-declared and what a host-earned row would require. Keep `FilteredVariant` immutable; public fields match other host evidence records (`ExecutionEnvironment`), while accessors remain a style preference if Tom wants parity with route result types rather than a consistency mandate. **Strongest counterpoint on fields:** the two fields are plain immutable evidence and public access is simpler; changing them now adds ceremony without changing invariants.

## Closes when

Tom accepts or revises each, the provisional-acceptance paragraph in the originating ticket is annotated with what superseded it, and the acceptance provenance — who, date, venue, relay source — is recorded.

## Graph maintenance

- Only Tom closes this. An agent may not convert a coordinator's provisional acceptance into a durable one by restating it.
- A revision that changes a class is an implementation change; file it rather than editing the landed record to match.

## Fact audit — 2026-08-10

Board status stays `awaiting-decision`: Tom has not answered the three questions for the provisional vocabulary, the originating provisional paragraph is not annotated, and partial prior acceptance of `UndispatchableDType` does not close this node. Repair corrected imprecise present-tense framing (unique node-less claim; "no accept-* ticket"; first-acceptance language for an already-accepted class; uniform-accessors mandate) without fabricating a supersession.
