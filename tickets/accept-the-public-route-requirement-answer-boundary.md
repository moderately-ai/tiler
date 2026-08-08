---
id: accept-the-public-route-requirement-answer-boundary
title: Accept or revise the public route-requirement answer boundary
status: awaiting-decision
priority: p1
dependencies: []
related: [land-the-backend-scoped-route-requirement-answer-adr, design-the-adapter-owned-route-requirement-answer-channel, accept-the-public-backend-provider-composition-boundary]
scopes: [contracts/foundation, contracts/decisions, contracts/integrations]
shared_scopes: [contracts/navigation, project/tickets]
paths: []
tags: [decision, needs-tom, public-boundary]
---
## User-visible outcome

Tom receives one evidence-backed packet for the seven public-boundary items ADR 0092 enumerated and did not accept, and no public surface conditional on them reaches the tree before he answers.

## Why this node exists

**Fact — the record was accepted with no acceptance node.** [ADR 0092](../docs/decisions/0092-answer-backend-scoped-route-requirements-in-the-owning-backends-vocabulary.md) reads `decision_status: accepted` at `:8`, and `:18` records "accepted by Tom on 2026-08-01 at the live review, together with the other decisions ratified that day". The acceptance landed at `f7e57bd` ("Close the acceptance pair; 0092 and 0093 are accepted with their sweeps executed"). ADR 0093's acceptance had a node — [`accept-adr-0093-cpu-vector-lane-tier`](accept-adr-0093-cpu-vector-lane-tier.md) — and released three implementation tickets. ADR 0092's did not, and none of its seven items has a ticket.

**Fact — the record says in terms that acceptance accepted none of them.** ADR 0092:20: "The research record lists **seven** public-boundary items under 'Public-boundary items, enumerated for Tom and not self-accepted', and accepting the model accepts **none** of them" — a new `pub mod` in `tiler-metal`'s crate root (the one item ADR 0075's mechanical categories fire for); reclassifying `tiler-metal` as a crate a consumer may name; the exact shapes of `observe_highest_gpu_family`, `decide_metal_route_requirement`, `AppleGpuFamilyConstant`, and `MetalRouteRequirementAnswer`; whether the observation crosses as a raw Apple constant at all; the minting constructor's shape; `MetalGpuFamilySupport` becoming a compatibility surface; and whether the governed key and version stay private to `tiler-metal`.

**Fact — item 2 is an amendment to an accepted contract sentence, and it has not happened.** ADR 0092:20 flags it as the sharpest instance: "the model now says the 'one crate a consumer names' sentence describes the non-dispatching consumer, and the act of amending `docs/architecture.md` to say so is item 2 of that list and has not happened." The sentence is at `docs/architecture.md` (anchor: `the one crate an inline-frontend consumer names`): "`tiler` is the one crate a consumer names, re-exporting `tensor!` and owning the absolute paths generated tokens spell." Under the accepted model that is true of a non-dispatching consumer and false of a dispatching one, so the contract as written now overstates.

> **The cited sentence has moved and has already been narrowed once, corrected 2026-08-04 by the stale-claim sweep at base `c4b4bdb9`. Whether that narrowing discharges item 2 is Tom's and is deliberately not decided here.** The sentence is at `docs/architecture.md` (anchor: `the one crate an inline-frontend consumer names`), not `:389`, and it no longer reads as quoted. It now reads: "`tiler` is the one crate an inline-frontend consumer names, re-exporting `tensor!` and owning the absolute paths generated tokens spell. This does not make `tiler` the accepted facade for consumers that construct and compile arbitrary semantic programs; the general graph and compiler surfaces currently remain in `tiler-ir` and `tiler-compiler`, and their eventual coherent facade is a separate public-boundary decision." Reproduce with `grep -n 'one crate an inline-frontend consumer names' docs/architecture.md`; the string this ticket quotes, `grep -n 'one crate a consumer names' docs/architecture.md`, now returns **no match** in that file. **So the flat overstatement this paragraph describes is gone**, and the Fact as written is refuted in its stated form. What is *not* established is that item 2 is discharged: the narrowing separates the inline-frontend consumer from the general-compiler consumer, and ADR 0092 item 2 is about a **dispatching** consumer being permitted to name `tiler-metal` — a distinction the amended sentence does not make. Two other sites still carry the old quotation and its `:389` citation as evidence for this item, in `docs/research/runtime/backend-scoped-route-requirement-answers.md:257` and `:317`; both are outside this node's scopes and are recorded here rather than edited. The packet must put the *current* sentence to Tom, state what the narrowing already settled, and ask only the residue.

**Fact — the measurement boundary is unusually wide and belongs in the packet.** ADR 0092:22 repeats it rather than leaving it one link away: "**nothing in this design was compiled or measured.** Every interface shape is a type-system reservation in ADR 0090's sense, none compiles, and the working implementation the record cites (`prototypes/candle-metal-adapter`) is an in-workspace crate reaching the vocabulary through an ordinary dependency, so it proves the decision logic and not the reachability this design exists for." A shape that has never compiled is a sketch to argue with, not a proposed interface.

## Ripens when

Set with Tom, 2026-08-01: this node ripens on **the first compiler-minted route requirement, or a dispatching consumer that needs the decoder** — whichever arrives first. Neither has: `grep -rn "RouteRequirement" crates/tiler-build/src/` returns nothing at base `0017345`, so the compiler mints no route requirement today, and no consumer dispatches. Until then the boundary is prospective, and putting seven unbuilt shapes to Tom would spend his time on interfaces the tree cannot yet exercise.

This is also why no separate implementation ticket exists for a facade-reachable answer to live-device route requirements: [`land-the-backend-scoped-route-requirement-answer-adr`](land-the-backend-scoped-route-requirement-answer-adr.md):56 states outright that implementation "is a separate phase decision under the implementation boundary and has no ticket, deliberately: research completion does not authorize scaffolding." Filing one now would pre-empt that phase decision. This node is the complete remedy.

## Decision boundary

This node is not research or implementation work. When it ripens, the packet presents each of the seven items with what it enables and prevents, its counterpoint, and a recommendation — one atomic question at a time, not a design dump. Item 2 goes with the exact proposed replacement text for `docs/architecture.md` (anchor: `the one crate an inline-frontend consumer names`), because the amendment is the acceptance act rather than a consequence of it.

## Closes when

Tom answers each of the seven items; `docs/architecture.md` (anchor: `the one crate an inline-frontend consumer names`) is amended or explicitly left; ADR 0092's status paragraph stops listing unaccepted items as outstanding; and any surface he accepts is released to its own implementation ticket rather than landed under this node.

## Ripeness check log

- 2026-08-07 — **not ripened.** Both named arrival conditions re-tested at base `ee858197` rather than relayed. The compiler mints no route requirement: `grep -rn "RouteRequirement" crates/tiler-build/src/` returns no match, unchanged from the `0017345` observation this node was filed on. No consumer dispatches: the producer that would supply the first minted requirement, [`emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable`](emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable.md), reads `status: deferred` and holds only an expired claim, so nothing is in flight toward it either. The seven items therefore stay prospective and are deliberately **not** put to Tom this cycle — the node's own reasoning is that unbuilt shapes spend his time on interfaces the tree cannot exercise, and that reasoning is intact. Recheck: `grep -rn "RouteRequirement" crates/tiler-build/src/` and `grep -m1 '^status:' tickets/emit-a-route-requirement-from-the-build-producer-so-a-family-authority-refusal-is-drivable.md`.
