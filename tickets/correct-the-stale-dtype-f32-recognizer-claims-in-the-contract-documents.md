---
id: correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents
title: Correct the stale dtype-f32 recognizer claims in the contract documents
status: in-progress
priority: p2
dependencies: []
related: [widen-the-strategy-recognizer-past-the-f32-wall, establish-bf16-optimizer-legality]
scopes: [contracts/navigation, contracts/numerics, contracts/optimizer]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, bf16, dtype, correction]
claimed_from: todo
assignee: w-dtype-docs
lease_expires_at: 1786159193
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
> **This bullet was true when written and is struck. Corrected 2026-08-07.** It read: "A BF16 region covering **several** occurrences is refused: `fusion_legality`'s capability table is keyed by the `f32` operation set, so the region's legality is `Unknown` and every cover placing it is ruled out. `establish-bf16-optimizer-legality` owns that." **That ticket landed on 2026-08-07** and the refusal is gone.

- A BF16 region covering **several** occurrences now **fuses**, under a proof carried at its own width with every obligation derived. The four reduction obligations are discharged **vacuously over an empty population**, and reassociation is withheld as `Unknown` rather than proved — so the correction must state the fusion as reachable *and* name those two boundaries, or it overshoots in the opposite direction from the text it replaces.
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


## Per-Fact audit at base `435bd0d5`, by the worker before editing

Every Fact above was re-read against source at this base. Verdicts:

- **"Five documents state a recognizer rule that no longer exists"** — **verified as to the documents, imprecise as to the counts.** `grep -rno 'dtype-f32' docs/ | sort | uniq -c` returns **ten** hits over those five files, not six: `docs/dtype-support.md` **four** (two on the semantic-signature paragraph, one on "What that did not buy", one on "What did not move"), `docs/roadmap.md` **four** (three in the reduced-precision-float row, one in the structural-limits paragraph), `docs/numerical-semantics.md` one, `docs/correctness-and-testing.md` two, `docs/compiler/optimizer.md` two. Both places this ticket says "three occurrences" of `dtype-support.md` are wrong by one.
- **The recognizer statement itself** — **verified.** `recognized_program_arithmetic` in `crates/tiler-compiler/src/request.rs` derives the program's one arithmetic type from its own values, returns `mismatch("dtype-recognized")` for an unspellable width and `mismatch("dtype-uniform")` for a mixed-width program, and `recognized_arithmetic` admits exactly `F32` and `Bf16`. No `dtype-f32` rule exists anywhere in `crates/`.
- **The two renamed tests** — **verified.** Neither old name appears in `cargo nextest list`; the successors are `a_flush_accepting_bf16_contract_reaches_a_selected_plan` and `the_accepted_bf16_contract_schedules_and_lowers_a_region_the_request_now_reaches`, both in `crates/tiler-compiler/tests/bf16_numerical_contract.rs`.
- **"A single-occurrence BF16 program … on a profile that dispatches the dtype and honours the contract"** — **verified but imprecise, and the imprecision is the whole surviving wall.** The profile must declare the *remaining consumable numerical dimensions* at this width, not merely dispatch it. `a_flush_accepting_bf16_contract_reaches_a_selected_plan` uses a test profile with `Bf16Rows::Complete`. The authoritative `BoundMetalCompileDeclaration::first_macos_apple9` declares BF16 dispatchability and the two subnormal tables and nothing else — every other numerical-dimension declaration in `crates/tiler-build/src/metal_declaration.rs`'s `first_macos_apple9` is bound to `ScalarArithmetic::f32()` — seven `declare_measured_*` calls over six dimensions, reassociation appearing twice for its two resolutions, plus contraction, permutation, signed zero, NaN assumptions, and infinity assumptions — so on that profile a flush-accepting BF16 contract clears the measured dimensions and meets contraction at disposition `Unknown`, and the target-local outcome is `NoFeasiblePlan`. `the_request_boundary_stops_at_the_ledgers_undeclared_bf16_contraction_row` (`crates/tiler-conformance`) is that observation against the ledger's own rows.
- **"reassociation is withheld as `Unknown` rather than proved"** — **false**, as the ticket's own trailing correction already records. `push_reduction_obligations` in `crates/tiler-compiler/src/fusion_legality.rs` discharges `ReductionReassociation` as `SoundProof` when `!has_reduction || reassociation == Forbidden`; a pointwise BF16 region satisfies the first disjunct vacuously. The `fusion_legality.rs:1641-1653` citation in that correction has drifted — the branch is now at `:1699`–`:1714`; the anchor `push_reduction_obligations` holds. The `bf16_numerical_contract.rs:691` citation resolves (the `fn` line).
- **"Three governed index-access lowering capabilities were added, one per registered BF16 family"** — **verified**: `governed_provider("constant-bf16" | "multiply-bf16" | "add-bf16")` in `crates/tiler-compiler/src/governed.rs`.
- **"`docs/compiler/optimizer.md` is done … its three in-correction mentions"** — **verified as to done, imprecise as to the count.** It carries **two**, both inside dated corrections that describe the rule as gone or replaced, and they are the worked example the closing rule names.
- **"`docs/dtype-support.md`'s three occurrences are still untouched"** — **imprecise**: four, all untouched at this base.
- **"`Physical carrier`'s qualifier 'schedule-assembled regions only' may now understate"** — **verified, and it does.** `boundary_carrier` is read by `derive_boundary_contract` on the frontier path, so any BF16 region that reaches a selected `PlanAlternative` through `compile()` has passed that derivation; the qualifier is widened to "request-reached and schedule-assembled regions" and the family note states what it still does not claim.

> **Correction, 2026-08-07 — the coordinator's "reassociation is withheld as `Unknown`" was over-general and is struck.** Found by the worker on [`correct-the-fusion-legality-wall-claims-left-in-the-compiler-after-bf16-legality-landed`](correct-the-fusion-legality-wall-claims-left-in-the-compiler-after-bf16-legality-landed.md), which declined to write the claim into the code rather than repeating it, and verified by the coordinator at `crates/tiler-compiler/src/fusion_legality.rs:1641-1653`.
>
> The obligation is discharged **`SoundProof`** when `!has_reduction || reassociation == Forbidden`. A multi-occurrence **pointwise** BF16 region has no reduction, so its `ReductionReassociation` records `SoundProof` **vacuously** — not `Unknown`. The `Unknown { "unproven-reassociation" }` branch requires a reduction **and** a permitting contract, which is precisely the surviving wall `a_contraction_permitting_bf16_contract_stops_at_the_fusion_legality_wall` (`crates/tiler-compiler/tests/bf16_numerical_contract.rs:691`).
>
> **The substance stands and only the mechanism was wrong:** reassociation is *not proved* for these regions, merely *not required*, because the region carries no reduction order to preserve. Say that, grounded on `BF16_FACT_REASSOCIATION_PERMITTED` being `false` and no BF16 family declaring an algebraic capability. **Writing "the obligation records `Unknown`" would be a new false claim** — the exact defect these tickets exist to remove.

## Per-hit classification after the corrections, 2026-08-07

`grep -rno 'dtype-f32' docs/ | wc -l` now returns **twenty** hits over the same five files, where it returned ten at the base, and **every one is inside a strike or a dated correction that describes the gate as retired**. The count rose because a correction quotes what it retires; the count is therefore not the check, and each hit was classified by its enclosing span:

| File and line | Hits | Classification |
| --- | --- | --- |
| `docs/compiler/optimizer.md` | 2 | Legitimate, unchanged, out of population — the worked example: both sit inside `*Corrected 2026-08-06/-08-07*` clauses naming the rule gone or replaced. |
| `docs/dtype-support.md` semantic-signature paragraph | 3 | Two were live claims, now struck in place; the third is inside the appended 2026-08-07 correction. The capability-table clause beside them was re-read and survives. |
| `docs/dtype-support.md` "What that did not buy" | 2 | One inside the struck sentence, one inside the correction. Struck clause by clause because two of its four clauses survive. |
| `docs/dtype-support.md` "What did not move, and why" | 2 | One inside the struck sentence, one inside the correction naming the renamed test. |
| `docs/dtype-support.md` new extent paragraph | 2 | Both describe the rule as retired, in the paragraph that states the row's reachable extent. |
| `docs/roadmap.md` reduced-precision-float row | 4 | Two inside struck sentences, two inside the corrections that retire them. |
| `docs/roadmap.md` structural-limits paragraph | 1 | Inside the struck refusal-list entry, which became two entries rather than being deleted. |
| `docs/numerical-semantics.md` | 1 | Inside the struck paragraph. |
| `docs/correctness-and-testing.md` "What that evidence bounds" | 2 | One struck, one inside the correction. |
| `docs/correctness-and-testing.md` "What that run crosses" | 1 | Inside the correction, quoting the retired attribution it replaces. |

No document states the gate as current behaviour. Every test name cited in the touched text resolves against `cargo nextest list`.
