---
id: accept-the-loader-variant-eligibility-vocabulary
title: Accept or revise the loader variant-eligibility vocabulary
status: in-progress
priority: p2
dependencies: [select-executable-variants-across-registered-backend-families, preserve-governed-key-types-in-loader-eligibility-diagnostics, make-loader-selection-refusal-formatting-total]
related: [expose-explicit-backend-provider-and-selection-policy-composition, decide-whether-a-loading-host-may-state-several-backend-families, accept-the-neutral-build-orchestration-boundary, bound-canonical-entry-ordinal-lookup-cost-in-loader-preflight]
scopes: [contracts/decisions, implementation/runtime]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [runtime, public-boundary, decision, needs-tom]
claimed_from: todo
assignee: worker-loader-eligibility
lease_expires_at: 1786664925
---
## User-visible outcome

The `tiler_runtime::load` variant-eligibility vocabulary rests on Tom's acceptance rather than on a coordinator's provisional overnight one, so the runtime half of the backend-composition surface stops resting solely on that provisional record.

**Correction — 2026-08-10.** Earlier wording said runtime was "the only one of the three that never reached him." Among the three composition-adjacent public surfaces, build orchestration is Tom-accepted (2026-08-05 under [`accept-the-neutral-build-orchestration-boundary`](accept-the-neutral-build-orchestration-boundary.md)); installed physical-provider and this loader vocabulary both still await Tom. Runtime is not uniquely node-less: this ticket is its accept node.

**Correction — 2026-08-13.** The clause "this loader vocabulary both still await Tom" is retired for this node. Tom accepted the vocabulary on 2026-08-11. The installed physical-provider node is a different ticket.

## Why this exists

**Fact — the surface was provisionally accepted by the coordinator and recorded for Tom, and no Tom decision yet supersedes that provisional vocabulary.** [`select-executable-variants-across-registered-backend-families`](select-executable-variants-across-registered-backend-families.md) closed `done` with a section headed "Provisional boundary acceptance (2026-08-01, overnight mode)" which states the coordinator accepted the loader vocabulary change and that it was "Recorded for Tom's morning review". No *other* `accept-*` ticket names that provisional review; **this ticket is the carrier node** for Tom's durable answer. Absence of a superseding Tom decision is not absence of a ticket. The three near-miss accept nodes — [`accept-the-public-route-requirement-answer-boundary`](accept-the-public-route-requirement-answer-boundary.md), [`accept-the-route-facts-dtype-dispatch-field`](accept-the-route-facts-dtype-dispatch-field.md), and [`accept-the-route-resource-requirement-spelling`](accept-the-route-resource-requirement-spelling.md) — are all about route *requirements* and none mentions variant eligibility.

**Correction — 2026-08-13.** The clause "no Tom decision yet supersedes that provisional vocabulary" is retired. Tom accepted the vocabulary on 2026-08-11; the originating paragraph is now headed "Provisional boundary acceptance (2026-08-01, overnight mode), superseded 2026-08-11" and names this ticket as the durable record. This node remains the carrier; the supersession it was filed to obtain has happened.

**Fact — the precedent says a provisional acceptance is superseded rather than left standing.** [ADR 0090](../docs/decisions/0090-compose-backends-per-responsibility-rather-than-per-backend.md), anchor `superseding the provisional overnight acceptance`, records that the build orchestration surface was accepted by Tom on 2026-08-05 at the live decision review under [`accept-the-neutral-build-orchestration-boundary`](accept-the-neutral-build-orchestration-boundary.md). The compiler surface has its node in [`accept-the-installed-physical-provider-public-surface`](accept-the-installed-physical-provider-public-surface.md). **Correction — 2026-08-11:** that node is now `todo`, not `awaiting-decision`: Tom accepted its four decisions and it awaits the mechanical rename and label sweep. This ticket is the corresponding accept node for the runtime loader eligibility vocabulary.

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

Accept removal of the three superseded broad classes and accept the three original ineligibility classes plus the selection-refusal split as coherent with the already-accepted `UndispatchableDType`: they preserve the filtered-versus-failed distinction and give each current refusal a different repair. Leave the already-accepted `UndispatchableDType` in place; do not file a duplicate gap ticket — [`declare-host-dtype-dispatchability-at-the-consumer-boundary`](declare-host-dtype-dispatchability-at-the-consumer-boundary.md) already records why the current frontend and Candle rows remain producer-declared and what a host-earned row would require. Keep `FilteredVariant` as leaf value-data with public fields, matching other host evidence records (`ExecutionEnvironment`), while accessors remain a style preference if Tom wants parity with route result types rather than a consistency mandate. **Correction — 2026-08-11:** an owned value with public fields is mutable; the earlier `immutable` wording was false. The relevant ADR 0074 question is instead whether the pair carries a cross-field verified invariant. It does not: it is read-transparent evidence that becomes meaningful in the loader-produced refusal which contains it.

## Accepted — 2026-08-11

**Decision.** Tom accepted the coordinator's ranked recommendation in the Codex coordination thread by replying `sounds good, accept`. The relay source is Tom's direct response in that thread. The routing semantics are accepted now; the ticket moves to `todo` because the exact public surface must land the two narrow correctness repairs below before its draft label can be retired.

**Correction — 2026-08-13.** Both named repairs have landed: [`preserve-governed-key-types-in-loader-eligibility-diagnostics`](preserve-governed-key-types-in-loader-eligibility-diagnostics.md) and [`make-loader-selection-refusal-formatting-total`](make-loader-selection-refusal-formatting-total.md) are `done`. What remains is documentation and draft-label retirement.

1. **One explicit execution environment per attempt.** The caller chooses exactly one target profile, backend family, executable representation, and dtype declaration before `preflight` or `prepare`. The loader may filter packaged variants only against that one atomic choice. It never retries another backend family, invents a default environment, or treats an absent dtype declaration as permission. A different backend requires a separate caller-controlled attempt before routing commit.
2. **Eligibility precedes applicability.** A host-ineligible variant is not a candidate and its guard is not evaluated. Among eligible variants, declared stable priority remains authoritative: a guard answering false advances to the next eligible member, while an unanswerable guard aborts rather than silently substituting a lower-ranked plan.
3. **The refusal split is accepted.** `NoEligibleVariant` means every packaged member was filtered by the stated environment. `NoApplicableVariant` means at least one eligible guard ran and every eligible guard answered false; it retains the reasons for members filtered before their guards. Collapsing these would erase opposite repairs.
4. **The fine-grained vocabulary is accepted.** `AssessedProfile`, `UnsupportedRepresentation`, `PayloadProfile`, and the previously accepted `UndispatchableDType` remain distinct. The superseded `UnexecutablePayload`, `IncompatibleTarget`, and `TargetDeclaration` remain removed; no deprecated aliases are retained in this unpublished pre-production workspace.
5. **`FilteredVariant` remains leaf value-data with public `variant` and `reason` fields.** It is not a verified product and carries no independent cross-field invariant. Public construction and mutation therefore do not mint a loader verification claim; the enclosing loader-produced refusal is what gives the pair its meaning.
6. **Governed identifiers remain typed in the accepted surface.** [`preserve-governed-key-types-in-loader-eligibility-diagnostics`](preserve-governed-key-types-in-loader-eligibility-diagnostics.md) replaces diagnostic `String` erasure with the existing `BackendKey`, `RepresentationKey`, and `TargetProfileKey` types across the newly accepted eligibility payloads and the directly coupled `TargetCompatibility`/dtype payloads. This changes no routing decision and has the same allocation order as cloning the underlying strings.
7. **Public refusal formatting is total.** [`make-loader-selection-refusal-formatting-total`](make-loader-selection-refusal-formatting-total.md) prevents externally constructible malformed `packaged`/`filtered` counts from reaching unchecked subtraction in `Display`. Loader-produced values already satisfy the count relationship; the repair makes the public value safe to format without trusting its origin.

**Runtime-performance boundary.** The accepted route remains a device-free host walk. It performs no kernel work and no automatic backend retry. The independent audit found repeated canonical-entry ordinal construction and lookup can make the host-side walk worse than the simple surface description suggests; [`bound-canonical-entry-ordinal-lookup-cost-in-loader-preflight`](bound-canonical-entry-ordinal-lookup-cost-in-loader-preflight.md) owns measuring and, if warranted, indexing that cost without blocking the correctness/public-surface repairs or changing selection semantics.

**Explicit exclusions.** This decision does not admit a set-valued execution environment, exact caller-selected variant ranks, automatic family fallback, fallback on an unanswerable guard, compatibility aliases for removed errors, a cost-based runtime selector, artifact identity changes, or a device probe during eligibility filtering.

## Closes when

The typed-key and total-formatting repairs land; the provisional-acceptance paragraph in the originating ticket remains annotated with what superseded it; the module documentation states the accepted included and excluded boundary rather than a draft; and targeted runtime tests plus repository publication gates pass.

## Graph maintenance

- Only Tom closes this. An agent may not convert a coordinator's provisional acceptance into a durable one by restating it.
- A revision that changes a class is an implementation change; file it rather than editing the landed record to match.

## Fact audit — 2026-08-10

Board status stays `awaiting-decision`: Tom has not answered the three questions for the provisional vocabulary, the originating provisional paragraph is not annotated, and partial prior acceptance of `UndispatchableDType` does not close this node. Repair corrected imprecise present-tense framing (unique node-less claim; "no accept-* ticket"; first-acceptance language for an already-accepted class; uniform-accessors mandate) without fabricating a supersession.

## Fact audit — 2026-08-13 at `4275c14b`

Every ticket Fact was re-read at this base before the close-out edit. The 2026-08-10 audit is **stale** as present-tense repository state.

1. **[FALSE as present-tense]** "Board status stays `awaiting-decision`" / "Tom has not answered the three questions." Status is `in-progress`. Tom accepted the ranked recommendation on 2026-08-11. The three questions stay as historical packet text; they are not open.
2. **[FALSE as present-tense]** "no Tom decision yet supersedes that provisional vocabulary" / "the originating provisional paragraph is not annotated." The originating Outcome paragraph is headed `Provisional boundary acceptance (2026-08-01, overnight mode), superseded 2026-08-11` and names this ticket. Verified by reading [`select-executable-variants-across-registered-backend-families`](select-executable-variants-across-registered-backend-families.md) at this base.
3. **[VERIFIED]** Both hard dependencies are `done`. `UnsupportedRepresentation` carries `BackendKey`/`RepresentationKey`; `UndispatchableDType::host_profile` and `TargetCompatibility::{ProfileKeyMismatch,DescriptorMismatch}` carry `TargetProfileKey`. `NoApplicableVariant` `Display` uses `checked_sub` and names an inconsistent public count rather than subtracting. `UnexecutablePayload`, `IncompatibleTarget`, and `TargetDeclaration` are absent from `crates/tiler-runtime`.
4. **[VERIFIED, already true]** The originating provisional paragraph was already annotated at this base. This close-out only retires the now-stale "keeps completion dependent" clause after those repairs landed.
5. **[FALSE as complete]** "the module documentation states the accepted included and excluded boundary rather than a draft." At this base the routing docs described the walk but did not name the accepted included/excluded set, and four sites still said `Labelled draft` under ADR 0075: `UnsupportedRepresentation`, `UndispatchableDType`, `TargetCompatibility::ProfileKeyMismatch`, and `TargetCompatibility::DescriptorMismatch`. Crate-level `lib.rs` still called the whole of `load` a reviewed draft with no eligibility carve-out.
6. **[VERIFIED]** The 2026-08-10 correction that "this loader vocabulary both still await Tom" is false for this node after 2026-08-11. The physical-provider node is a different ticket.
7. **[VERIFIED]** Surface inventory at `750b29e0` remains a dated snapshot. The live types still match the Added/Changed/Removed/Unchanged census, with the two later repairs (typed keys, total `Display`) applied.

## Scope addition

`implementation/runtime` was added with `tkt set --add-scope` before `crates/tiler-runtime` module docs were edited. The labelled-draft markers lived only there; retiring them without the scope would have been a silent escape. `prototypes/serial-sum-run` is in that glob and was not edited.
