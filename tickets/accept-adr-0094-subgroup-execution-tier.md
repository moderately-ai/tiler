---
id: accept-adr-0094-subgroup-execution-tier
title: Accept or reject the subgroup execution tier ADR
status: done
priority: p2
dependencies: [land-the-subgroup-execution-tier-adr]
related: [design-the-subgroup-execution-tier, compose-the-two-level-subgroup-and-workgroup-reduction]
scopes: [contracts/decisions]
shared_scopes: [project/tickets]
paths: []
tags: [contracts, adr, scheduling, subgroup, decision]
---
## User-visible outcome

The subgroup execution tier's ADR moves from `proposed` to `accepted` or is rejected, which is what releases the implementation tickets gated behind it.

**Only Tom closes this ticket.** No agent may set it `done`, and no agent may do its work. Its permanent status is `awaiting-decision`; an agent that finds it in `todo` should set it to `awaiting-decision` and do nothing else.

**The id is a placeholder.** [`land-the-subgroup-execution-tier-adr`](land-the-subgroup-execution-tier-adr.md) renames it to `accept-adr-NNNN-subgroup-execution-tier` once the number is fixed.

## The genuine choices, stated so Tom can act without re-deriving them

The nine decision items are not equally consequential. Six follow from primary specifications and have no surviving alternative; three encode priorities and are the real questions.

- **Item 4 — only the ascending-mask butterfly is admitted, and the descending-stride tree is statable and refused.** The descending form is the idiom Apple's own specification prints and the shape most GPU reduction code takes. Refusing it means a planner may not copy the vendor example. The derivation is that its leaf order is bit-reversed, so it consumes contributor permutation on top of reassociation at identical instruction count — but a reader who expects the familiar shape will experience this as Tiler refusing standard practice.
- **Item 6 — identity injection is required in the general case rather than optional.** This is what makes the lane identity a mandatory checked field with a proof obligation, and it is the largest new correctness surface the tier adds. It follows from the width being imposed rather than chosen, so the alternative is not "make it optional" but "refuse every reduction whose contributor count does not divide the subgroup width" — which would refuse most reductions.
- **Item 7 — the width resolves across three stages, adding a `PreparedKernelPreflight` gate no other schedule has.** The alternative is trusting a compile-profile declaration alone, which is cheaper and is wrong on Metal, where `threadExecutionWidth` is a prepared-pipeline property. The cost of the correct answer is a preflight stage with routing-commit ordering obligations.

## What acceptance does and does not do

Acceptance flips `decision_status` to `accepted`, sets `adopted_by` on the research record and moves its `disposition` to `adopted`, updates both catalog views, and releases the implementation tickets. It registers nothing, declares no target profile row, emits no shuffle, and admits neither the two-level composition nor the narrowing tree — both of which the ADR explicitly excludes.

## Closes when

Tom accepts or rejects it.

## Outcome — accepted (2026-08-01)

**Tom accepted the subgroup execution tier ADR at the live review on 2026-08-01**, relayed by the coordinator. The decision is settled; the acceptance *execution* is separately queued, so this ticket stays open until that landing rather than closing on the decision alone. Its remaining work is therefore the sweep this ticket's own "What acceptance does" section already enumerates — flip `decision_status` to `accepted`, set `adopted_by` on the research record and move its `disposition` to `adopted`, update both catalog views, and rename this ticket to `accept-adr-NNNN-subgroup-execution-tier` per the placeholder note above — not a further decision.

**The three implementation tickets this node claims to release now exist.** At acceptance they did not: [`design-the-subgroup-execution-tier`](design-the-subgroup-execution-tier.md):65 lists the four tickets that design filed and none is an implementation ticket, so the claim at `:15` and `:31` that acceptance "releases the implementation tickets gated behind it" released nothing. They were filed on 2026-08-01 and each depends on this node in the `todo`-gated-by-a-parked-acceptance idiom the workflow configuration describes, so this node becoming `done` releases them structurally:

- [`admit-subgroup-bindings-into-the-schedule-vocabulary`](admit-subgroup-bindings-into-the-schedule-vocabulary.md)
- [`admit-subgroup-typed-values-and-collectives-into-the-kernel-ir`](admit-subgroup-typed-values-and-collectives-into-the-kernel-ir.md)
- [`declare-metal-subgroup-realization-facts-in-the-target-profile`](declare-metal-subgroup-realization-facts-in-the-target-profile.md)

**Unchanged by acceptance.** The two-level composition and the narrowing tree stay excluded, as `:31` states; the nine public-boundary items enumerated at [the subgroup execution tier](../docs/research/scheduling/subgroup-execution-tier.md):334 remain unaccepted and come to Tom individually at implementation time; and [`correct-the-subgroup-threads-route-dimension-meaning`](correct-the-subgroup-threads-route-dimension-meaning.md) is independent of this decision, as `design-the-subgroup-execution-tier.md:65` records.

## Closed 2026-08-01 — the acceptance landed as ADR 0094

**Fact — what is recorded.** Tom accepted the subgroup execution tier's model at the live review on 2026-08-01, and the execution landed [ADR 0094](../docs/decisions/0094-bind-a-subgroup-combine-to-a-register-transfer-tree.md) — "Bind a subgroup combine to a register-transfer tree with a stated stride order and a proved lane identity" — with `decision_status: accepted`. **What he accepted is the model — the ADR's nine numbered decisions — and none of the nine public-boundary items** the research record enumerates; those are unchanged and still arrive under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md) with the implementation tickets below.

**The provenance of that acceptance, named rather than dressed up.** The decision reached this ticket through the filing wave that wrote the Outcome section above, and reached the execution branch through the coordinator's dispatch brief — the ordinary channel, and the one every "Tom accepted" record in this repository came through. **If the relay was wrong the repair is bounded and mechanical:** `decision_status` back to `proposed` in ADR 0094, its two catalog rows back to `proposed`, the research record's `disposition` and `adopted_by` back to `pending`, this node back to `awaiting-decision`, and the four dependents back behind it. Nothing beyond that was released — no crate, test, fixture, or spike was touched, and the one contract sentence rewritten under this acceptance ([the artifact ABI](../docs/artifact-abi.md)'s subgroup-width relation) states a correction that had already landed on its own authority at `77c36d5`.

**Four tickets depend on this node, not three, and closing it makes three of them ready.** The Outcome above names three implementation tickets; [`compose-the-two-level-subgroup-and-workgroup-reduction`](compose-the-two-level-subgroup-and-workgroup-reduction.md) is a fourth dependent and was not counted. All four sit in the `todo`-gated-by-a-parked-acceptance idiom, so no status edit was needed — `awaiting-decision` never satisfied a dependent and `done` does. Verified against the board rather than asserted: `tkt ready` named no subgroup ticket before this closure and names [`admit-subgroup-bindings-into-the-schedule-vocabulary`](admit-subgroup-bindings-into-the-schedule-vocabulary.md), [`declare-metal-subgroup-realization-facts-in-the-target-profile`](declare-metal-subgroup-realization-facts-in-the-target-profile.md), and [`compose-the-two-level-subgroup-and-workgroup-reduction`](compose-the-two-level-subgroup-and-workgroup-reduction.md) after it. [`admit-subgroup-typed-values-and-collectives-into-the-kernel-ir`](admit-subgroup-typed-values-and-collectives-into-the-kernel-ir.md) declares a second dependency on the schedule-vocabulary ticket and correctly stays out of `ready` until that lands — the same asymmetry [`accept-adr-0093-cpu-vector-lane-tier`](accept-adr-0093-cpu-vector-lane-tier.md) had to correct, where the node claimed three releases and delivered two.

**Closed by:** [`land-the-subgroup-execution-tier-adr`](land-the-subgroup-execution-tier-adr.md), which carried the ADR file, both catalog rows, the research record's disposition move, the proposal-era sweep, and this closure. The rule that only Tom closes this node governed it for its whole life and is preserved unedited above; what changed is that the decision was taken, not that the rule relaxed.

## Current board correction — 2026-08-09

The closure-time ready-set account above is historical. ADR 0094's model remains accepted and its implementation remains not started. [`admit-subgroup-bindings-into-the-schedule-vocabulary`](admit-subgroup-bindings-into-the-schedule-vocabulary.md) and [`declare-metal-subgroup-realization-facts-in-the-target-profile`](declare-metal-subgroup-realization-facts-in-the-target-profile.md) are now `awaiting-decision` at their separate ADR 0075 public boundaries. [`admit-subgroup-typed-values-and-collectives-into-the-kernel-ir`](admit-subgroup-typed-values-and-collectives-into-the-kernel-ir.md) remains `todo` but is dependency-blocked by the schedule-vocabulary decision. The fourth dependent, [`compose-the-two-level-subgroup-and-workgroup-reduction`](compose-the-two-level-subgroup-and-workgroup-reduction.md), is `done`, and ADR 0096 accepted its representation. Acceptance released these questions from ADR 0094; it did not accept their later public spellings.
