---
id: carry-the-restored-ladder-point-into-the-four-records
title: Carry the restored ladder point into the four records the explain-ceiling fix could not edit
status: in-progress
priority: p2
dependencies: []
related: [refuse-nothing-legal-on-the-explain-detail-ceiling, carry-the-widened-ladders-corrections-into-the-four-records, region-expansion-exhaustion-loses-the-only-feasible-plan]
scopes: [research/artifacts, contracts/decisions, contracts/artifacts, contracts/foundation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, measurement]
claimed_from: todo
assignee: agent-orch-docs
lease_expires_at: 1786075458
---
## User-visible outcome

The four records that state the identity-growth ladder's domain and its walls say what the 2026-08-06 post-explain-ceiling measurement established, so a reader of any of them reaches the ten-point ladder and its one remaining wall rather than the nine-point ladder and a defect that no longer exists.

## Why a carrier ticket

[`refuse-nothing-legal-on-the-explain-detail-ceiling`](refuse-nothing-legal-on-the-explain-detail-ceiling.md) removed the eleven-operation `InvalidCompilerOutput` wall and re-derived the ladder. It held `implementation/compiler`, `project/tickets`, and `research/program-planning`, so it corrected the spike itself and [`docs/research/program-planning/complete-model-ingestion-and-execution.md`](../docs/research/program-planning/complete-model-ingestion-and-execution.md), which it owned. The four records below are under `research/artifacts`, `contracts/decisions`, `contracts/artifacts`, and `contracts/foundation`. This ticket is their carrier, exactly as [`carry-the-widened-ladders-corrections-into-the-four-records`](carry-the-widened-ladders-corrections-into-the-four-records.md) was for the same four records after the previous ladder change.

## The measurement the corrections carry

**Measurement, 2026-08-06, `spikes/program-planning/identity-growth`, Apple M4 Max, macOS 27.0 `26A5388g`, toolchain `nightly-2026-07-19`, retained at `results/2026-08-06-post-explain-ceiling-apple-m4-max-macos27.0-26A5388g/growth.tsv`.**

- The ladder is **ten points, 2..=11 operations**, not nine points over 2..=10.
- `program_bytes(n) = 3525n + 727` and `graph_bytes(n) = 134n + 149`, **residual 0 at all ten points**. No coefficient moved, so **every numeric conclusion downstream of the curve is confirmed unchanged**: the 19,038-operation refusal point, the 148/149 embedding-ceiling crossing, 219,277 bytes and 41.8% at the governed budget of 62, P2's 180,502 bytes and ×372, and the whole-model 3,765,427 bytes.
- The rows at 2..=10 reproduce the previous run's structural columns **byte for byte**, which is the measurement that the compiler fix moved no identity.
- **The out-of-domain confirmation these records recorded as lost now exists again**, and this is the one statement that inverts rather than shifts. `3525·11 + 727 = 39,502` was a prediction about a program the compiler refused; when the refusal was removed the program compiled to 39,502 bytes exactly. It is a check the fit could have failed and did not. It is also one step wide, and it is now inside the domain, so a reader must not read it as a bound on the extrapolation to 19,038.
- **The eleven-operation `InvalidCompilerOutput` wall is gone and was a defect, not a bound.** The compiler's coverage-gap explain rule emitted one record per (cover, region) pair; cover enumeration reached about 2,300 of them against a single unimplemented singleton region and exhausted the trace's canonical-byte ceiling. The rule now emits one record per unimplemented region with a `blocked-covers` count.
- **`region_expansions` still binds at twelve**, unchanged, and remains filed as [`region-expansion-exhaustion-loses-the-only-feasible-plan`](region-expansion-exhaustion-loses-the-only-feasible-plan.md). It is now the *only* wall between the ladder and the governed budget that is not about program size.

## The four records and the exact statements that move

- **`docs/artifact-abi.md` line 247** (`contracts/artifacts`) — "over the widened 2..=10 ladder … residual zero at all nine points — with no out-of-domain probe available, because the path refuses this family above ten operations for reasons that are not program size" → **2..=11, residual zero at all ten points, with one out-of-domain confirmation at eleven operations now inside the domain, and the path refusing above eleven.** Its numeric claims (`3525n + 727`, 19,038, 148/149, 219,277, 41.8%) are all confirmed and none moves.
- **`docs/ir.md` line 1138** (`contracts/foundation`) — the same phrasing, the same replacement, and the same confirmation of its numbers.
- **`docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md`** (`contracts/decisions`) — no number moves. Two statements are superseded: the header measurement paragraph's "2..=10 operations with four class-checked walls at 11, 12, 62, and 63 … residual zero at all nine points" and its closing sentence naming the explain detail ceiling as an open defect; and "Bounds on the evidence", whose recorded weakening — "no out-of-domain confirmation at all" — is **discharged**. Record the discharge as such, with its own bound: the confirmation is one step wide and now in-domain. The Context and derivation body keeps its accepted-tense pre-fold figures, as an ADR's derivation must.
- **`docs/research/artifacts/manifest-fixed-content-growth.md`** (`research/artifacts`) — four places carry "all nine points of its 2..=10 domain", "refuses this family above ten operations", "no out-of-domain confirmation available at all", and "**No program above 10 operations compiles at all** … 11 refuses on the explain authority's detail ceiling and 12..=62 on `region_expansions` exhaustion, so … both walls are filed as defects" (Sections at lines ~159, ~165, ~169, ~209, ~211, ~241). Every one moves to the ten-point ladder, the single remaining wall, and the restored out-of-domain point. Its ordering conclusion — the embedding ceiling binds first — is unaffected.

## Closes when

All four records carry the corrections, each read in full around the edit so no adjacent sentence still asserts the superseded state, and no record still names the explain detail ceiling as an open defect.
