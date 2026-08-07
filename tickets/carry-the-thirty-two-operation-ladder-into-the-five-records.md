---
id: carry-the-thirty-two-operation-ladder-into-the-five-records
title: Carry the thirty-two-operation ladder and its changed wall class into the five records the region-expansion fix could not edit
status: in-progress
priority: p2
dependencies: []
related: [region-expansion-exhaustion-loses-the-only-feasible-plan, carry-the-restored-ladder-point-into-the-four-records, carry-the-widened-ladders-corrections-into-the-four-records, widen-the-identity-growth-ladder-to-the-governed-operation-budget]
scopes: [research/artifacts, contracts/decisions, contracts/artifacts, contracts/foundation, research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, measurement]
claimed_from: todo
assignee: agent-orch-docs
lease_expires_at: 1786080145
---
## User-visible outcome

Every record that states the identity-growth ladder's domain and its walls says what the 2026-08-07 measurement established, so a reader of any of them reaches the thirty-one-point 2..=32 ladder and a `BudgetExhausted` wall at thirty-three rather than the ten-point ladder and a `NoFeasiblePlan` wall at twelve that no longer exists.

## Why a carrier ticket

[`region-expansion-exhaustion-loses-the-only-feasible-plan`](region-expansion-exhaustion-loses-the-only-feasible-plan.md) removed the twelve-operation wall and re-derived the ladder. It held `implementation/compiler`, `contracts/optimizer`, `research/program-planning`, and `project/tickets`, so it corrected the compiler, the two optimizer contracts, and the spike itself. The records below are under `research/artifacts`, `contracts/decisions`, `contracts/artifacts`, and `contracts/foundation` — and one under `research/program-planning` that a concurrent live claim held while the fix landed, so it is carried here rather than edited across a claim boundary. This ticket is their carrier, exactly as [`carry-the-restored-ladder-point-into-the-four-records`](carry-the-restored-ladder-point-into-the-four-records.md) was after the previous ladder change.

## The measurement the corrections carry

**Measurement, 2026-08-07, `spikes/program-planning/identity-growth`, Apple M4 Max, macOS 27.0 `26A5388g`, toolchain `nightly-2026-07-19`, retained at `results/2026-08-07-post-coverage-extremes-apple-m4-max-macos27.0-26A5388g/growth.tsv`.**

- The ladder is **thirty-one points, 2..=32 operations**, not ten points over 2..=11.
- `program_bytes(n) = 3525n + 727` and `graph_bytes(n) = 134n + 149`, **residual 0 at all thirty-one points**. No coefficient moved, so **every numeric conclusion downstream of the curve is confirmed unchanged**: the 19,038-operation refusal point, the 148/149 embedding-ceiling crossing, 219,277 bytes and 41.8% at the governed budget of 62, P2's 180,502 bytes and ×372, and the whole-model 3,765,427 bytes.
- The rows at 2..=11 reproduce the previous run's structural columns **byte for byte**, which is the measurement that the compiler fix moved no identity: widening which regions a search reaches does not change the identity of the plan it selects.
- **The twenty-one points 12..=32 were each a prediction about a program the compiler refused, and every one landed on the fitted line to the byte.** That is the strongest confirmation this curve has had. It is also spent: all of those points are now *inside* the domain, so the extrapolation to 19,038 again has **no out-of-domain confirmation at all**, and the records that currently say it has one (the eleven-operation point at 39,502 bytes) must say it had one and that the ladder consumed it.
- **The wall changed both its position and its class.** 12..=62 refused `NoFeasiblePlan` on `region_expansions`; 33..=62 now refuse `BudgetExhausted` on `region_members`, raised after planning with the trace sealed and `budget-stop:region-members:32:33` in it. 63 still refuses `BudgetExhausted` on `semantic_operations`, before any target-qualified trace. Records that name the wall's class, its bound, or "12..=62" are all stale.
- **`region_members = 32` is the new binding constraint, and it is a bound on region size rather than program size.** For a pointwise family the recognized partition is the whole program and nothing smaller is implementable, so the whole-program region is the only cover with a plan and 32 is the widest such program this profile can plan. Whether that bound should move with `semantic_operations` is Tom's, recorded as the parked question on the parent ticket; no record should assert either answer.

## The known stale sites

Each was found by `grep -rn "region_expansions\|12\.\.=62\|2\.\.=11"`; read each document in full before editing, because the surrounding conclusions differ.

- **`docs/research/artifacts/manifest-fixed-content-growth.md`** (`research/artifacts`) — Sections 5, 6, and 8 carry "all ten points of its 2..=11 domain", the one earned out-of-domain confirmation, "12..=62 still refuse on `region_expansions`, filed as a defect", and "**No program above 11 operations compiles at all**". Every one moves. Its ordering conclusion — the embedding ceiling binds first — is unaffected.
- **`docs/research/program-planning/complete-model-ingestion-and-execution.md`** (`research/program-planning`) — the L1 relay paragraph naming the `region_expansions` wall among its six relayed measurements, the "**12 through 62 refuse `NoFeasiblePlan`**" measurement paragraph, and the superseded-note beneath it that ends "`region_expansions` still binds at twelve, and this record's decoder-layer program is still not compilable". The decoder layer is still not compilable and the reason is now a different bound.
- **`docs/artifact-abi.md:247`** and **`docs/ir.md:1138`** (`contracts/artifacts`, `contracts/foundation`) — the shared measurement sentence states the 2..=11 domain, residual zero at ten points, the earned out-of-domain confirmation, and "the path refuses this family above eleven operations on `region_expansions`, which is not about program size".
- **`docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md:20`** (`contracts/decisions`) — "2..=11 operations with class-checked walls at 12, 62, and 63", "residual zero at all ten points", and "12..=62 still refuse on `region_expansions` exhaustion, filed as a defect". The fold's own figures are all confirmed by the wider ladder and none of them moves.

## Checks

`tkt lint`, and `grep -rn "region_expansions" docs/` returning only statements that are true of the current tree.
