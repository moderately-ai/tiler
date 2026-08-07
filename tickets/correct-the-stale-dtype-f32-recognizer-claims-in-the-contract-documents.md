---
id: correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents
title: Correct the stale dtype-f32 recognizer claims in the contract documents
status: todo
priority: p2
dependencies: []
related: [widen-the-strategy-recognizer-past-the-f32-wall, establish-bf16-optimizer-legality]
scopes: [contracts/navigation, contracts/numerics, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, bf16, dtype, correction]
---
## What is false

**Fact, at the merge of `widen-the-strategy-recognizer-past-the-f32-wall`.** Five documents state a recognizer rule that no longer exists. The exact check is `grep -rn 'dtype-f32' docs/`:

- `docs/dtype-support.md` (three occurrences, including the BF16 support-matrix narrative and "**A BF16 program still does not compile**")
- `docs/roadmap.md`
- `docs/numerical-semantics.md`
- `docs/correctness-and-testing.md`
- `docs/compiler/optimizer.md`

`select_supported_strategy` no longer carries a `dtype-f32` rule. It derives the program's one arithmetic type and admits the two widths this build spells a per-point body in; a width it cannot spell is refused under `dtype-recognized` and a mixed-width program under `dtype-uniform`.

**Fact.** `docs/dtype-support.md` cites two compiler tests by name that were renamed with the rule: `a_flush_accepting_bf16_contract_reaches_the_recognizer_dtype_wall` and `the_accepted_bf16_contract_schedules_and_lowers_a_region_the_request_cannot_reach`.

## What is true now, stated precisely so a correction does not overshoot

- A **single-occurrence** BF16 program is recognized, planned, and reaches a selected `PlanAlternative` under a contract of its own width on a profile that dispatches the dtype and honours the contract.
- A BF16 region covering **several** occurrences is refused: `fusion_legality`'s capability table is keyed by the `f32` operation set, so the region's legality is `Unknown` and every cover placing it is ruled out. `establish-bf16-optimizer-legality` owns that.
- Three governed index-access lowering capabilities were added, one per registered BF16 family, so "no lowering capability" is false.
- Nothing here says BF16 *executes* end to end through `compile()`; that run is a separate ticket.

## The support-matrix row

`docs/dtype-support.md`'s BF16 row is the one this work advances, and it should say what moved and what did not: the compile path is reachable for a one-occurrence program; optimizer legality is not; and the conformance run through `compile()` is still owed.

## Why it is filed rather than fixed

`docs/**` is owned by `contracts/navigation`, `contracts/numerics`, and `contracts/optimizer`; the recognizer branch held `implementation/compiler` only.

## Required evidence

- No document claims a `dtype-f32` recognizer rule.
- Every cited compiler test name resolves against the tree the correction lands on.
- The BF16 support-matrix row names both what became reachable and what did not, each with the authority that decides it.

## Closes when

> **This closing condition was unsatisfiable and is replaced. Corrected 2026-08-07 by the coordinator.** It read: "`grep -rn 'dtype-f32' docs/` is empty, …". **That can never be true**, because these documents' own established convention is to **quote the retired text inside a dated correction** — the 2026-08-04, -08-05 and -08-06 corrections in `docs/compiler/optimizer.md` all do it, and three `dtype-f32` mentions now live there legitimately for exactly that reason. A closing condition that demands the repository forget what it corrected is the mirror of the unfireable check: a check that can never say *yes*. Found by the worker on [`correct-the-recognizer-era-sentences-in-the-optimizer-contract`](correct-the-recognizer-era-sentences-in-the-optimizer-contract.md) while doing the work this ticket also covers.

**Closes when** every remaining `dtype-f32` mention in `docs/` is either **inside a dated correction that describes the retired gate as retired**, or gone; no document states the gate as current behaviour; every cited test name exists; and the BF16 row states the reachable extent and its remaining boundary.

The distinction is the whole check, so make it mechanically: for each hit, the enclosing paragraph must be a dated correction or the hit is a live claim. Report the classification per hit rather than a bare count — a count cannot tell the two apart, which is how the original condition went wrong.

## Its own body is partly stale — repair before dispatch

Two things this ticket asserts have been overtaken, both reported by workers rather than found by a scan:

- **"What is true now" says optimizer legality is unreachable** and points at `establish-bf16-optimizer-legality` as its owner. That ticket **landed on 2026-08-07**: a multi-occurrence BF16 region now fuses under a proof at its own width, with every obligation derived, the four reduction obligations discharged vacuously over an empty population, and reassociation explicitly withheld as `Unknown`.
- **`docs/dtype-support.md`'s three occurrences are still untouched**, but the rest of that file moved on 2026-08-07 under [`move-the-bf16-optimizer-legality-ledger-cell`](move-the-bf16-optimizer-legality-ledger-cell.md) — which also found two cells *understated* rather than overstated and corrected them. Read the file's current state, not this ticket's description of it. That work also flagged that **`Physical carrier`'s qualifier "schedule-assembled regions only" may now understate**, since a single-occurrence BF16 program reaches a selected plan; deciding that is this ticket's, under its "reachable extent" obligation.

`docs/compiler/optimizer.md` is **done** — it was corrected in full on 2026-08-07 and is no longer part of this ticket's population, though its three in-correction mentions are the worked example of the classification rule above.
