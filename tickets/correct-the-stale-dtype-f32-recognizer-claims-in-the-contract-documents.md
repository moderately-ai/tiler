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

`grep -rn 'dtype-f32' docs/` is empty, every cited test name exists, and the BF16 row states the reachable extent and its remaining boundary.
