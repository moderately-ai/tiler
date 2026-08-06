---
id: decide-whether-the-l3-ladder-rung-moves-on-the-dispatched-contraction-cell
title: Decide whether the L3 ladder rung moves on the dispatched contraction cell
status: done
priority: p2
dependencies: []
related: [integrate-the-contraction-vertical-into-the-runtime, publish-an-l3-contraction-cell-through-the-accepted-route, raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells, record-the-contraction-execution-row-and-correct-the-matrix-headline, spike-first-metal-contraction-vertical]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [navigation, roadmap, contraction, maturity]
---
## Why this exists

**Fact.** The L3 rung of the language-model inference ladder ([`docs/roadmap.md`](../docs/roadmap.md), `### The ladder`) names the capability "One contraction runs end to end on Metal", and its `Maturity today` cell still reads "nothing compiles or executes" — a description of what the [realization record](../docs/research/scheduling/first-metal-contraction-realizations.md) delivered, which is a research outcome rather than the capability.

**Fact — the capability the row names has since been delivered, twice, at two different extents.** [`integrate-the-contraction-vertical-into-the-runtime`](integrate-the-contraction-vertical-into-the-runtime.md) is `done`: on 2026-08-02 a `td,od->to` contraction ran end to end through the accepted AOT and runtime route at `[2, 3] x [2, 3] -> [2, 2]`, five operand cases bit-compared against the reference-evaluated sidecar, with exact `MTLCommandBufferStatusCompleted` before readback. [`publish-an-l3-contraction-cell-through-the-accepted-route`](publish-an-l3-contraction-cell-through-the-accepted-route.md) is `done`: on 2026-08-05 `w_decode_kv` at `1 x 1024 x 1024` — one of L3's *own* six correctness cells, and the pinned workload's decode `k_proj`/`v_proj` shape — ran through the same route and the SHA-256 of its executed result bytes matched the realization probe's retained `direct` value. [`raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells`](raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells.md) is `done` and made all six cells reach a selected physical plan.

**Fact — no navigation ledger recorded any of this until 2026-08-06.** The maturity audit of 2026-08-06 found the roadmap's contraction row still asserting "no execution row exists", and its own dispatch premise was narrower than the evidence: the audit believed the six L3 cells compiled but had not dispatched, when one had. [`record-the-contraction-execution-row-and-correct-the-matrix-headline`](record-the-contraction-execution-row-and-correct-the-matrix-headline.md) corrected the operation-family matrix row, the matrix headline, the two ladder clauses, and `docs/status.md`'s device-execution bullets, and deliberately left the L3 rung's own cell unmoved — recording the delivery in the ladder's prose beneath the table and filing this ticket — because promoting a rung is a maturity judgement about the ladder rather than a correction to a stale sentence.

## The decision, and why it is not obvious

The rung's wording is satisfied on its face: one contraction does run end to end on Metal, at one of the rung's own profile cells. Three things argue against reading that as the rung delivered, and they are what this ticket has to weigh rather than assume:

- **One cell of six, one host row, one realization.** The dispatched cell is `w_decode_kv` under the `direct` realization. The [L3 record](../docs/research/scheduling/first-metal-contraction-realizations.md) selects `tiled` on cost, and [`realize-the-tiled-contraction-schedule-and-its-metal-emission`](realize-the-tiled-contraction-schedule-and-its-metal-emission.md) is `deferred`. A rung that reads as delivered while the realization its own record selected is unbuilt is a claim the next reader will over-trust.
- **Every rung so far fired on record delivery, not capability delivery**, and L5, L6, and L8's triggers were fired on that reading with the derivation recorded in their cells. Moving L3 on *capability* introduces a second, incompatible reading of the same column unless the ladder says which one each rung used.
- **The consequence is not local.** L4 and L7's activation triggers name "L3 delivers"; both already fired on the design-rung reading, so nothing unblocks — but the maturity vocabulary the rest of the table uses would no longer be uniform, and the `Maturity today` column is what a reader consults before claiming the ladder's state.

## Closes when

The L3 row's `Maturity today` cell either states the delivered capability with its exact boundary (cell, host row, realization, and what is not covered) and the column's two readings are distinguished wherever the ladder uses them, or states in the row itself why the delivery is recorded beneath the table rather than in the cell — and in both outcomes the ladder's prose, the [operation-family support matrix](../docs/roadmap.md#operation-family-support-matrix) contraction row, and `docs/status.md`'s device-execution bullets agree with each other. Nothing here changes any measurement; the evidence is fixed and already recorded.

## Non-goals

Re-running or widening any measurement; the remaining five L3 cells; a second host row; the `tiled` realization; and any change to L4, L5, L6, L7, or L8's own cells beyond the vocabulary consistency the decision forces.

## Decided 2026-08-06 — the rung holds, with a named re-evaluation trigger

**Tom decided at the live session's decision round, relayed and executed by the coordinator:** the L3 `Maturity today` cell does not move on the one dispatched cell. The three arguments this ticket carried stand: one cell of six; the unselected `direct` realization while `tiled` is the record's cost-based selection; and the column's design-rung reading, which a capability-based move would fork. **Re-evaluation trigger, any of:** all six L3 correctness cells dispatched with retained-digest matches; the `tiled` realization lands and dispatches; or a deliberate re-read of the ladder column's semantics (its own ticket, if ever filed). The delivery remains recorded in the ladder prose and the contraction row, which is where a reader finds it today.
