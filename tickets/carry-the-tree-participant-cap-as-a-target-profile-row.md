---
id: carry-the-tree-participant-cap-as-a-target-profile-row
title: Carry the tree participant cap as a target profile row
status: in-progress
priority: p2
dependencies: []
related: [cap-the-tree-reduction-participants-at-the-measured-256]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [target-profiles, public-boundary]
claimed_from: todo
assignee: coord
lease_expires_at: 1786174094
---
## Per-Fact audit at `cc667626`, 2026-08-08

Re-read at this base before any edit, per AGENTS.md. Verdicts below; the bodies that follow are the corrected text.

| Ticket Fact | Verdict | Evidence |
| --- | --- | --- |
| The constant's doc says a second profile must not inherit it | **verified** | `crates/tiler-compiler/src/physical.rs`, anchor `second target profile should carry its own row`. The ticket's rendered-sentence anchor spans a line break and greps as absent; the single-line fragment is the durable one. |
| `capped_tree_partition` takes no target and `single_workgroup_tree_region` does not consult the profile for the width | **verified** | Signature `fn capped_tree_partition(contributors: u64)`; the caller reads `request` only for the numerical contract. |
| `single_workgroup_tree_region` is its **sole** caller | **imprecise** | Sole *production* caller. `three_strategy_domain` in `crates/tiler-compiler/src/target.rs` also calls it, inside `#[cfg(test)] mod tests`. It matters: that helper asks the rule as a *target-independent* predicate, so a profile-parameterized rule has to give it a profile. |
| The doc asserts a downstream crate's declaration and a repository-wide inventory | **verified, and understated** | Three inventory claims, not two: `declares 32,768 bytes`, `no profile in this repository`, and `the widest workgroup any profile in this repository declares`. |
| The `declare_*` / `declare_measured_*` pair must be built by this ticket | **false — the mechanism landed** | `activate-measured-reduction-selection-from-a-target-cost-row` is `done`. `CostRow` is a private enum in `target.rs` with one public pair plus one reader per row, `TargetCostRowResolution::Unknown` meaning *no preference*, a conditional descriptor section behind `COST_ROW_DOMAIN`, and `declare_measured_*` taking `TargetCompileProfileMeasurementSource`. This ticket is an instance of that mechanism, not a design. |
| The "Reserved" section's premise that the sibling is parked | **false** | It is `done`; Tom accepted the *model* on 2026-08-07 and expressly excluded the spelling. |
| `BoundMetalCompileDeclaration::first_macos_apple9` is in reach | **false** | It is `crates/tiler-build/src/metal_declaration.rs` = `implementation/build`, which this ticket does not declare. |

## The gap between what the code does and what its own doc says it should

**Fact — the constant's doc says a second profile must not inherit it.** `MEASURED_TREE_PARTICIPANT_CAP` (`crates/tiler-compiler/src/physical.rs`) documents itself as a property of one host row — one profile, one contract, one program family, `f32`, and not established for another Apple family, OS row, dtype, or device — and states that a "second target profile should carry its own row rather than inherit this one".

**Fact — the code makes inheritance mandatory and silent.** `capped_tree_partition(contributors: u64)` takes no target. Its sole production caller `single_workgroup_tree_region` receives a `VerifiedTargetRequest` that carries the profiles and does not consult it for the width. There is no typed row, no `Unknown` fallback, and nothing fails when a second profile is added: the Apple9 number governs it silently. The defect this names is real and is unchanged by the findings below.

**Inference — this is the failure direction AGENTS.md names.** Hardware is to be modelled "through typed target profiles, properties, schedule alternatives, feasibility predicates, and cost models", and the compiler core is to stay independent of any one consumer's facts. Today the doc states the right architecture and the code states the opposite, and only the code runs. The defect is latent while one profile exists and becomes a wrong width the moment a second arrives.

**A smaller instance of the same leakage, worth fixing with it.** `capped_tree_partition`'s doc, in compiler core, asserts facts about a downstream crate's declaration and about the repository's whole profile inventory — three of them: that the measured row `declares 32,768 bytes`, that `no profile in this repository` sits in the affected band, and that `the widest workgroup any profile in this repository declares` is the qualified Apple9 entry's 1,024. All are true today and nothing fails if a profile outside them is added. Core documentation should not depend on a consumer-side inventory it cannot see. **This deliverable is independent of the decisions below** — it is a doc repair in `implementation/compiler` that touches neither the value, the rule, nor selection — but it is not free-standing either, because two of the three claims are load-bearing for the correctness argument the section below dismantles, and the third sits in the paragraph an open sibling ticket closes on. Repair it by relocating authority, not by deletion: core may state the arithmetic bound it owns, and whether a *given* profile admits that width is the feasibility authority's answer.

## What this ticket owes

- The participant preference declared on the target profile as a **second `CostRow` variant** plus its own `declare_*` / `declare_measured_*` pair and reader, additively — the mechanism `activate-measured-reduction-selection-from-a-target-cost-row` landed and whose documentation already states that this is how a second row arrives. The measured constructor carries a `TargetCompileProfileMeasurementSource`, so validity stays `MeasuredEnvironment` and cannot widen into a portable claim.
- `BoundMetalCompileDeclaration::first_macos_apple9` declaring it from the retained 2026-08-07 calibration, citing that spike. **Needs `implementation/build`, which this ticket does not declare.**
- **Silence meaning "no preference", never "no plan"** — and a profile declaring no row must remain plannable. What silence should *select* is an open fork, recorded below; the ticket's original wording picked one branch without pricing it.
- The identity consequence carried completely. **Bounded by reading at this base:** `cost_rows` is written into `complete_descriptor` behind `COST_ROW_DOMAIN`, in a section emitted only when non-empty, so a profile declaring no row keeps its bytes and `COMPLETE_PROFILE_DESCRIPTOR_DOMAIN` stays at `v11`. Only the qualified Apple9 profile declares a cost row today, so only its canonical descriptor moves — and it does move, since the section grows a second entry. Every pinned artifact identity and cache subject derived from that descriptor must be recomputed on the merged tree and ledgered, with a before/after table.

## Blocked 2026-08-08: parameterizing the cap lets a cost input decide feasibility

**Measurement — exhaustive over `0..40_000` contributors, on a temporary parameterization of `capped_tree_partition` that was reverted rather than committed.** The rule's chosen width tracks `2 * cap - 2`, so the cap's *value* is what bounds it. Columns: counts the rule admits; counts whose chosen width `tiler_ir::schedule::workgroup_tree_tile` refuses, which is the tree **withdrawn**; the widest width reached; and counts where the rule's decline set stops agreeing with `governed_partition`.

| declared cap | admitted | withdrawn by the tile | widest | first withdrawal | decline set moved |
| --- | --- | --- | --- | --- | --- |
| 0 | 39,996 | **39,996** | 1 | 4 → 1 participant | **4,201**, first at the prime 5 |
| 1 / 2 / 64 | 35,795 | 0 | 199 | — | 0 |
| **256 (today)** | 35,795 | 0 | 509 | — | 0 |
| 512 | 35,795 | 0 | 1,021 | — | 0 |
| 1,024 | 35,795 | 0 | 2,039 | — | 0 |
| 2,048 | 35,795 | 0 | 4,093 | — | 0 |
| 4,096 | 35,795 | **9,936** | 8,179 | 8,194 → 4,097 | 0 |
| 100,000 | 35,795 | **22,256** | 19,999 | 8,194 → 4,097 | 0 |

**A declared zero is the worst case, and it is the one value the sibling row explicitly admits.** `declare_saturated_parallel_fold_steps` documents that "A value of zero is admitted and is a statement rather than an absence". At a cap of zero this rule's fallback branch starts its search at `ceiling + 1 = 1`, and every count is a multiple of one, so it returns a **one-participant** partition — at every count from four up, and at 4,201 *prime* counts that admit no partition at all. `workgroup_tree_tile` refuses one participant as the semantically redundant barrier, so the tree is withdrawn everywhere, and the invariant the whole design rests on — that `capped_tree_partition`'s decline set is exactly `governed_partition`'s — is violated.

**Above `2,048` the withdrawal is ordinary rather than degenerate.** A cap of 4,096 reaches 8,179 participants, past `MAX_COOPERATIVE_PARTICIPANTS`, so the tree is withdrawn at 9,936 counts below 40,000 where it is offered today. That is exactly what ruled out `max(capped_tree_partition, governed_partition)` in [`bound-the-tree-cap-s-unmeasured-downward-direction`](bound-the-tree-cap-s-unmeasured-downward-direction.md): a width preference that withdraws a legal alternative has decided feasibility.

**The band is narrower still against a profile's own declared width.** The qualified Apple9 entry declares 1,024 threads per workgroup, and the widest width stays inside it only to a cap of **512**. So the admissible band is `2 <= cap <= 512` for that profile and `2 <= cap <= 2_048` for the IR representation limit — and the tighter bound is a function of *another row on the same profile*, which the builder does no cross-family validation against, and which is `Unknown` on a sparse profile that declares no workgroup-threads axis at all.

**Why this blocks rather than costs a guard.** The repository's stated correctness argument for the rule being a preference is an argument from the literal 256: `capped_tree_partition`'s doc says the arithmetic ceiling "is what keeps the rule a preference rather than a feasibility decision", and `pipeline::tests::the_tree_widens_toward_the_cap_rather_than_truncating_at_it` asserts `chosen <= 2 * cap - 2` and pins the widest reachable width at 509 exactly. Parameterize the cap by profile and that argument does not survive; restoring it needs either a build-time bound coupling a cost row to a capability axis that may be `Unknown` — giving a cost row a hard-error mode, the inversion the family exists to avoid — or accepting that a declared row withdraws a strategy. Both are Tom's calls, not an implementation detail.

## Also unresolved: what silence means, where the ticket contradicts itself

The ticket owes "a profile that declares no row must select exactly as the balanced rule does". The sibling mechanism's own precedent is the opposite: `declare_saturated_parallel_fold_steps` documents that a profile declaring nothing "selects exactly as it did before this row existed, byte for byte". For this row those two readings are **different behaviours**, and neither is free:

- **Silence keeps the cap at 256.** Nothing changes for a silent profile, so the Apple9 number still governs every other profile — the defect this ticket exists to fix is not fixed, only made declarable.
- **Silence falls back to `governed_partition`.** The leakage is genuinely fixed, but this changes the tree's width on every profile that does not declare the row, which the ticket's own non-goals forbid as a selection change; nothing measured says the balanced rule is right for an unknown device either; and on a silent profile the tree and the split would read one rule again, which is the condition [`separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ`](separate-the-tree-and-split-groupings-at-a-contributor-count-where-their-partitions-differ.md) records as its *unfired* state.

## The evidence rung the row would carry, which the move must not launder

`MEASURED_TREE_PARTICIPANT_CAP`'s value is **empirical evidence on one host** in the upward direction only. The rule around it — that a count nearer the cap is cheaper — is **`Unknown`** at every count the rule moves, because no measured shape has a non-power-of-two contributor count and the two formulations coincide on powers of two. A target profile row reads as a measured device fact, and the measured constructor would stamp it `MeasuredEnvironment`. Carrying an unmeasured rule hyperparameter into that structure promotes the claim by relocation. If the row is ever built, its documentation must say that the *value* is fitted on one host and the *rule* it parameterizes is unmeasured — the row is a fitted hyperparameter of Tiler's width rule, not a machine quantity like `cost.saturated-parallel-fold-steps`, which at least names something about the device.

## Reserved

**This would add a `pub` `TargetProfileBuilder` declaration pair and a reader, and move an identity domain, so Tom accepts the exact surface.** Its mechanism is settled: a second `CostRow` variant plus its own pair and reader, additively, exactly as `CostRow`'s own documentation says a second row lands. What is *not* settled is whether the row should exist at all, given the two sections above. Before any code, Tom decides: (1) whether a width preference may be profile-declared at all when doing so can withdraw a legal alternative, and if so which guard is acceptable; and (2) what silence means. Question (1) gates (2).

## Explicit non-goals

No change to the cap's value or to the rule's shape — [`bound-the-tree-cap-s-unmeasured-downward-direction`](bound-the-tree-cap-s-unmeasured-downward-direction.md) landed the rule and is `done`. No selection change. No new measurement; the excursion width is [`measure-the-tree-width-excursion-past-the-cap`](measure-the-tree-width-excursion-past-the-cap.md)'s.

## Scope the ticket does not declare

`BoundMetalCompileDeclaration::first_macos_apple9` is in `crates/tiler-build/` = `implementation/build`, which is absent from this ticket's `scopes`. Whoever takes the work after the decisions above needs it added. The doc-leakage deliverable also overlaps an open sibling: [`pin-the-local-memory-refusal-band-the-tree-cap-opened`](pin-the-local-memory-refusal-band-the-tree-cap-opened.md) closes on `capped_tree_partition`'s band paragraph ceasing to describe the band as argued-not-observed, which is the same paragraph carrying two of the three inventory claims. Sequence them rather than racing.

## Graph maintenance

Filed 2026-08-07 by the coordinator from the post-landing multi-lens audit of [`cap-the-tree-reduction-participants-at-the-measured-256`](cap-the-tree-reduction-participants-at-the-measured-256.md), which recorded the constant's bound honestly in prose and could not build the seam within its scope.

2026-08-08, worker at base `cc667626`: audited every Fact, repaired two false ones and one imprecise one, and stopped before any `crates/` edit. The stop is not a scope or estimate judgement — it is the finding above, that carrying this particular quantity as a profile row lets a declared cost value withdraw a legal alternative, which AGENTS.md forbids and which the rule's own tested correctness argument depends on not happening. Recommend `awaiting-decision` until Tom answers the two questions in **Reserved**; the coordinator owns the state change. The reproduction is a temporary parameterization of `capped_tree_partition` to `(cap, contributors)` plus an exhaustive sweep of `0..40_000` per cap, comparing the chosen width against `tiler_ir::schedule::workgroup_tree_tile` and the decline set against `governed_partition`; it was reverted rather than committed, and `git status` was clean afterwards.
