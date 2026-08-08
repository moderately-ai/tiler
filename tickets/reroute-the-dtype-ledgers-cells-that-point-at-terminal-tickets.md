---
id: reroute-the-dtype-ledgers-cells-that-point-at-terminal-tickets
title: Reroute the dtype ledger's BF16 and host-dtype cells off terminal tickets
status: in-progress
priority: p2
dependencies: []
related: []
scopes: [contracts/navigation, research/numerics]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, dtype, bf16, work-graph]
claimed_from: todo
assignee: w-reroute-t
lease_expires_at: 1786166309
---
## Three passages route a reader to owners that are all `done`, two of them for work that was ruled out

Verified 2026-08-07 at base `7c371155` by `verify-and-file-the-remaining-maturity-audit-leads`. Two independent defects in one file plus its research counterpart, filed together because they are the same class, the same file, and one exclusive scope.

## Defect 1 — BF16's remaining rungs are routed to an owner set that is entirely terminal

**Fact — the routing sentence.** The BF16 Trigger closes `docs/dtype-support.md "BF16's remaining rungs are carried by the live tickets named in the paragraphs above"` and in the research tracks record under `#### D-4`.

**Fact — D-4's own eight owners are all `done`.** `docs/research/numerics/dtype-family-research-tracks.md "Remaining owners, by rung."` names exactly eight: `admit-bf16-into-the-schedule-and-kernel-vocabulary`, `admit-the-bf16-type-and-carrier-into-every-total-map`, `carry-bf16-through-the-artifact-encoding-and-identity`, `establish-bf16-optimizer-legality`, `lower-bf16-to-metal`, `validate-bf16-at-the-runtime-routing-boundary`, `conform-the-bf16-vertical-end-to-end`, and `state-and-check-a-bf16-numerical-contract`. Reproduce:

```sh
for t in admit-bf16-into-the-schedule-and-kernel-vocabulary admit-the-bf16-type-and-carrier-into-every-total-map \
         carry-bf16-through-the-artifact-encoding-and-identity establish-bf16-optimizer-legality lower-bf16-to-metal \
         validate-bf16-at-the-runtime-routing-boundary conform-the-bf16-vertical-end-to-end \
         state-and-check-a-bf16-numerical-contract; do grep -m1 '^status:' tickets/$t.md; done
```

All eight return `status: done`. Every one of the thirty-five tickets linked from anywhere in `docs/dtype-support.md` that concerns BF16 is likewise `done`. D-4's heading `docs/research/numerics/dtype-family-research-tracks.md "Owner: the live BF16 track; no new ticket."` is stale in the same motion: the track is not live.

**The narrow claim holds and the broad reading does not — check this before widening the repair.** A scan of tickets referenced from `docs/dtype-support.md` finds **16** non-terminal ones, so the file as a whole is not routing into a void. Every one of the 16 is an integer, quantized, sub-byte, execution-only, or other-family owner — `admit-a-storage-carrier-for-integer-program-inputs` at `blocked`, eleven `deferred` track owners, three quantized `todo`s, and `state-the-non-enumerable-float-conformance-profile` at `deferred`. **None is BF16.** The defect is confined to the BF16 owner set; do not restate it as a claim about the ledger's owners generally.

**The conclusion survives, and the remaining rungs are real.** This is not "BF16 is finished, delete the sentence". D-4 itself records what is genuinely outstanding and unowned: the two ADR 0091 conversion families are registered in neither direction, and their keys and spellings remain Tom's under ADR 0075. The reference-evaluation paragraph's conversion and accumulator clauses are untouched by every landing. So the sentence names real remaining work and points at a set of closed tickets for it — the worst of the two failure modes, because the reader concludes it is owned.

**Note the same Trigger's other half is fine.** `state-the-non-enumerable-float-conformance-profile` owns the `f16/f64/f128` remainder and is `deferred`, which is a live owner. Do not sweep it.

## Defect 2 — two cells say "until X lands" for a ticket that landed and recorded the opposite

**Fact — the two cells.** In the IEEE `f32` section, `docs/dtype-support.md "so the filter is tautological on exactly those paths until"` `declare-host-dtype-dispatchability-at-the-consumer-boundary` "derives the row from a bound device". In the BF16 section, `docs/dtype-support.md "so it is tautological on those paths until"` the same ticket "lands".

**Fact — it landed, and its Outcome rules the expected fix out.** `tickets/declare-host-dtype-dispatchability-at-the-consumer-boundary.md` is `done`, and its Outcome opens: `tickets/declare-host-dtype-dispatchability-at-the-consumer-boundary.md "that part is structural, and the ticket asked for it to be said rather than fixed"`. Its `Surviving restatements` section states that the facade path can reach no observation at all, because `execution_environment` builds the `ExecutionEnvironment` that is an *input* to adapter construction, so no device exists to consult; and that the Candle prototype's gap "is a decision to record and not a task to schedule". A reader following either cell arrives at a closed ticket expecting a fix that was deliberately ruled out.

**The conclusion versus its ground — and a careless repair breaks this.** The tautology claim itself is **still true**: both consumer paths do restate the producer's declaration, and the landed ticket confirms it rather than removing it. What is false is only the routing and the expectation — "until X lands" for something that landed and could not. **Do not delete the tautology sentences.** Reroute them: the one place a host-earned row can arise is an integration's `RuntimeAdapter::bind_execution_context`, which holds the device the facade does not, and the decision is recorded in `docs/integration/frontends.md` under **Direct byte embedding**. That contract's own reference to the ticket is correctly written in the past tense and needs no change.

**Sibling sweep, so this is not re-raised.** Every `until <ticket>` construction in `docs/` was extracted and its target's status read: six distinct tickets, all `done`. Four are past-tense records of work that landed and are correct as written; the fifth, the Q-SEM-015 planning gate in `docs/open-questions.md`, explicitly states "All three are `done`, so the gate is open". The two cells above are the only live-tense pointers of this class in the repository.

## Requirements

- Reroute the BF16 Trigger sentence and D-4's owner block to what is actually outstanding and who holds it — for the conversion families, Tom under ADR 0075 — rather than to eight closed tickets. Both files must move together; correcting one leaves the other forking the same claim.
- Reroute the two host-dtype cells to `bind_execution_context` and to `docs/integration/frontends.md`, keeping the tautology claim, which is still accurate.
- Leave `state-the-non-enumerable-float-conformance-profile` and the 16 non-BF16 owners alone.
- Prefer a searchable anchor to a line number in anything added; `make citations` covers `docs/**`.

## Closes when

No BF16 passage in either file routes a reader to a terminal ticket for outstanding work; the genuinely unowned BF16 remainder is named with the authority that holds it; the two host-dtype cells keep their tautology claim and point at the boundary that could discharge it; the non-BF16 owners are untouched; and `make citations` is green.
