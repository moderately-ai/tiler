---
id: activate-measured-reduction-selection-from-a-target-cost-row
title: Activate measured reduction selection from a target cost row
status: done
priority: p1
dependencies: [calibrate-and-activate-parallel-reduction-selection]
related: [calibrate-device-cost-models, implement-parallel-reduction-strategies]
scopes: [implementation/compiler, implementation/build, contracts/optimizer, research/target-profiles, research/program-planning]
shared_scopes: [project/tickets, contracts/navigation]
paths: []
tags: [implementation, cost-model, target-profiles, selection]
---
## User-visible outcome

Physical selection prefers a parallel reduction where the qualified profile's own measured evidence says it is faster, and the serial fold where it says the opposite, with explain output naming the term that decided.

## Why this is parked rather than todo

**Correction — 2026-08-10.** This section is the pre-acceptance filing posture. Frontmatter is `status: done`; the activation landed and integrated 2026-08-07 (see Outcome / Integrated). It is **not** live board state and must not be read as `awaiting-decision` or parking against the closed ticket.

**Tom decides two things this ticket cannot execute without.** It adds a `pub` `TargetProfileBuilder` declaration for a quantity no target currently carries, and moving that row moves the canonical descriptor, which moves every pinned artifact identity and cache subject derived from it. Both are reserved: consequential public boundaries and identity-domain steps are his, and this one is a *new kind* of row rather than another instance of an existing kind.

~~It is `awaiting-decision` rather than `deferred` because nothing is missing.~~ **Correction — 2026-08-10.** That present-tense board claim is retired; the ticket closed `done` after landing. The measurement, model, held-out score, and design below were the complete filing-time basis that made the work landable in one commit once Tom accepted the direction.

## The evidence this rests on

**Measurement, 2026-08-07** — [`spikes/program-planning/reduction-dispatch-crossover`](../spikes/program-planning/reduction-dispatch-crossover/README.md), retained at `results/2026-08-07-apple-m4-max-macos27.0-26A5388g/`, on a host matching the [authority ledger](../docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md)'s offline and execution rows in every field.

- The serial fold costs up to **50.7x** the best parallel plan (4 rows of 8,192 contributors) and as little as **0.56x** it (16,384 rows of 4). Both ends are far outside the noise.
- The two parallel strategies are inside each other's noise almost everywhere, so **the decision is binary**: parallelize or not.
- A three-parameter work-span model — `sum over stages of ( encoder + max(work / parallel_threads, depth) * step )` — fitted on perfect-square contributor counts agrees with the measured verdict on **24 of the 26 held-out cells whose verdict is separated**, worst measured penalty **1.81x**.
- **Only `parallel_threads` moves a decision.** Scaling `encoder` by twenty or `step` by a tenth leaves every predicted winner unchanged; scaling `parallel_threads` by a quarter drops held-out agreement to 20 of 26 and the worst penalty to 3.04x.

So the row this ticket needs is **one number**: the fold steps the device retires at once when saturated. Fitted at 1.056e3 on that host, and determined to roughly a factor of four — quadrupling it left fit-set agreement unchanged and *improved* the held-out worst penalty to 1.20x.

## The design problem to settle first, because it is not the row's value

**A cost row is not a capability key, and the profile vocabulary is built for capability keys.** [`docs/research/program-planning/flash-class-capability-set.md`](../docs/research/program-planning/flash-class-capability-set.md) already eliminated putting a bandwidth or clock number on a target profile, and the argument applies unchanged here: every `CapabilityAxis` variant is a *hard bound*, silence about one is `Unknown`, and `Unknown` never reaches an executable frontier. A cost row declared the same way would make silence render a profile **unexecutable for a quantity no feasibility predicate reads**, which is the wrong failure direction. Silence about a cost term must mean "no preference", not "no plan".

Second, `crates/tiler-compiler/src/component_cost.rs` records why a second cost-model key cannot simply join the first: `PlanStructuralCost::dominates` returns `false` across differing model keys, so plans carrying different keys never dominate each other, the non-dominated set silently becomes the whole set, and Pareto pruning goes dark with nothing reporting it.

Third, selection is a Pareto relation over exact structural counts with a canonical-identity tie break, deliberately **not** a scalar latency total order (`crates/tiler-compiler/src/pipeline/planning.rs`, `select_non_dominated`). A measured cost term that decides between mutually non-dominated plans is a change to what selection *is*, not a new dimension in what it already does.

## Implementation keys

Settle the three above and then, in one commit:

- declare the term on `TargetProfile` through a `declare_*` / `declare_measured_*` pair whose measured constructor carries the same `TargetCompileProfileMeasurementSource` the grid-axis row uses, so its validity stays `MeasuredEnvironment` and cannot widen to a portable claim;
- have `BoundMetalCompileDeclaration::first_macos_apple9` declare it from the retained 2026-08-07 measurement, citing that spike;
- thread it to the point where reduction alternatives are compared, and make the comparison explain itself — the winning alternative's explain row must name the term and both sides of the `max`, not merely report `selected`;
- recompute the canonical descriptor length, the standard Metal artifact identity, and its cache subject on the merged tree, enumerating each moved pin;
- keep hard feasibility untouched: no infeasible plan may become an expensive one, and a profile that declares no cost row must select exactly as it does today.

Do not widen `PlanStructuralCost` with a latency dimension as a shortcut. Its four dimensions are exact counts a plan carries; a fitted quantity beside them would make a Pareto relation over measured and counted quantities at once, and a profile without the row would then dominate differently from one with it.

## Required evidence

The selection change is mutation-proved on the term: perturbing the declared value changes the selected alternative on a named shape, or refuses. A profile declaring no term selects bit-identically to today, proved by an unchanged golden. The explain report names the deciding term. Every moved identity pin is enumerated with its before and after. The measured shapes the new selection prefers are the ones the retained sweep measured faster, checked against the retained TSV rather than re-argued.

## Closes when

Selection consults the declared term, the qualified profile declares it from retained measurement, explain names why the winner won, no infeasible plan is represented as expensive, identity moves completely in one commit, and Tom has accepted the exact public surface.

## Graph maintenance

- Keep after `calibrate-and-activate-parallel-reduction-selection`, which supplies the measurement and the model this ticket activates.
- `calibrate-device-cost-models` remains the owner of general analytical-cost calibration. This ticket is one term for one decision and must not be widened into that.
- If the design problem above resolves against a profile row, file the alternative carrier and close this rather than reshaping it silently.

## Accepted — 2026-08-07

**Tom approved the direction on 2026-08-07** in the coordination session, witnessed first-hand by the coordinator, on the basis presented: a measured cost row is admitted, **declared as a distinct kind from a capability axis**, with silence about it meaning *no preference* rather than *no plan*.

**The acceptance came with a standing instruction, and it governs how this ticket is executed:** do not cut scope or decisions for short-term gain. Performance, correctness, long-term maintainability, code quality, and compatibility are all to be weighed — a cheaper shape that defers one of them is not a saving. Nothing in the list below may be dropped, narrowed, or split off without saying so explicitly and giving the reason.

### What the acceptance settles

- The **direction**: selection may consult a measured term where the qualified profile declares one.
- The **carrier kind**: a cost row is *not* a `CapabilityAxis`. Declaring it as one would make silence render a profile unexecutable for a quantity no feasibility predicate reads, which is the wrong failure direction. The flash-class capability record already eliminated that shape for a bandwidth number and the argument transfers unchanged.
- The **silence rule**, which is testable rather than aspirational: **a profile declaring no cost row must select bit-identically to today, proved by an unchanged golden.**

### What the acceptance does not settle, and must not be quietly assumed

The exact public spelling of the `declare_*` / `declare_measured_*` pair remains a public boundary under ADR 0075 and comes back to Tom with the built surface. Acceptance of the model is not acceptance of its spelling.

### Obligations carried forward in full

Every item below was already in this ticket and is restated because the acceptance instruction forbids trimming them:

1. Declare the term through a `declare_*` / `declare_measured_*` pair whose measured constructor carries the same `TargetCompileProfileMeasurementSource` the grid-axis row uses, so its validity stays `MeasuredEnvironment` and cannot widen into a portable claim.
2. `BoundMetalCompileDeclaration::first_macos_apple9` declares it from the retained 2026-08-07 measurement, citing that spike.
3. Thread it to the point where reduction alternatives are compared, and **make the comparison explain itself**: the winning alternative's explain row names the term and both sides of the `max`, not merely `selected`.
4. Recompute the canonical descriptor length, the standard Metal artifact identity, and its cache subject **on the merged tree**, enumerating each moved pin. Two branches moved these same three pins on 2026-08-07 from different bases and neither's values survived; the current values are artifact identity `23c46a19…`, cache subject `e89c4d82…`, fixed content 64,542 bytes.
5. Keep hard feasibility untouched: no infeasible plan may become merely an expensive one.
6. **Do not widen `PlanStructuralCost` with a latency dimension as a shortcut.** Its four dimensions are exact counts a plan carries; a fitted quantity beside them would make a Pareto relation over measured and counted quantities at once, and a profile without the row would then dominate differently from one with it.
7. Two constraints that are correctness rather than taste, and that the implementation must answer rather than route around: `PlanStructuralCost::dominates` returns `false` across differing model keys, so plans carrying different keys never dominate each other, the non-dominated set silently becomes the whole set, and **Pareto pruning goes dark with nothing reporting it**. And selection today is a Pareto relation over exact structural counts with a canonical-identity tie break, deliberately *not* a scalar latency total order — a measured term that decides between mutually non-dominated plans is a change to what selection **is**, not a new dimension in what it already does.

### Evidence required, unchanged

The selection change is mutation-proved on the term: perturbing the declared value changes the selected alternative on a named shape, or refuses. A profile declaring no term selects bit-identically to today. The explain report names the deciding term. Every moved identity pin is enumerated with before and after. The shapes the new selection prefers are checked against the retained TSV rather than re-argued.

### If the design resolves against a profile row

This ticket's own instruction stands: file the alternative carrier and close this rather than reshaping it silently.

## Outcome — 2026-08-07

Landed on `tkt/activate-measured-reduction-selection-from-a-target-cost-row` from base `fe3ad943`. Every obligation of the Accepted section is discharged; nothing was dropped, narrowed, or split off. One scope was added — `research/program-planning` — because the required "checked against the retained TSV" evidence is a rerunnable checker that belongs beside the spike it scores.

### The design question the acceptance did not settle, and the fact that settled it

**The measured term ranges over the retained *valid* plans, not over the non-dominated view, because on this program family that view is a singleton.** The cheap shape — a better tie break inside the non-dominated set — cannot express the retained measurement at all, and this was measured rather than argued: `pipeline::tests::the_parallel_reduction_plans_are_structurally_dominated` compiles the reduction family at one row of 4,096 contributors, asserts the portfolio holds all three strategies, and asserts `SelectedPortfolio::non_dominated()` holds exactly one. The frontier-level statement of the same fact was already in the tree: `the_frontier_retains_the_split_beside_the_serial_reduction` records that the split "is worse on every structural dimension, so it can never win by pruning. Preference is `activate-measured-reduction-selection-from-a-target-cost-row`'s."

So the measured term can prefer a **structurally dominated** plan. That is a change to what selection *is*, and it is stated in `crates/tiler-compiler/src/measured_cost.rs`, in `select_non_dominated`'s own documentation, and in the optimizer contract rather than absorbed. What licenses it: structural dominance never claimed to prove a plan faster — the optimizer contract's own words are that its policy key "makes no latency claim" — and the retained sweep refutes fewer-resources-is-faster by up to 50.7x on a named contour.

**Both correctness constraints are answered rather than routed around.**

1. **`dominates` across model keys.** No second cost-model key is ever minted. `PlanStructuralCost` is untouched — four exact dimensions, one key, unchanged `dominates` — `aggregate_cost` is untouched, the frontier's single-key check is untouched, and `SelectedPortfolio::non_dominated` still computes and explain still reports the same Pareto view for every alternative. `measured_cost` has no `dominates` for one to be written with, exactly as `component_cost` has none. Pareto pruning cannot go dark because nothing new enters it.
2. **Pareto versus a scalar total order.** Answered explicitly rather than absorbed, above and in the module header. `PlanStructuralCost` was **not** widened with a latency dimension.

### The activated model, and the one thing about it that had to be measured

```text
fold_steps = sum over stages of max( work, depth * P )
```

`P` times the fitted work-span model at `encoder = 0, step = 1`. Dropping `step` is *provably* order-preserving — one positive factor over the whole sum. Dropping `encoder` is not provable that way, and the argument for it is that `encoder` prices dispatch count, which `PlanStructuralCost` already carries as an exact dimension and prunes on, so pricing it here would put one quantity under two authorities.

**Provably order-preserving and measured-inert are different claims, so the second was measured.** [`spikes/program-planning/reduction-dispatch-crossover/activated_selector_check.py`](../spikes/program-planning/reduction-dispatch-crossover/activated_selector_check.py) scores the reduced selector on the retained TSV's own recorded `threads:work:depth` triples:

```text
92 cells, 276 measured alternatives
P fitted ( 1056.0)  fit       separated cells  33/36   worst measured penalty 1.17x at 1024 x 1024
P fitted ( 1056.0)  held-out  separated cells  26/29   worst measured penalty 1.81x at 1024 x 128
P x 0.25 (  264.0)  fit       separated cells  29/36   worst measured penalty 1.68x at 1024 x 16384
P x 0.25 (  264.0)  held-out  separated cells  22/29   worst measured penalty 3.04x at 256 x 128
P x 4    ( 4224.0)  fit       separated cells  34/36   worst measured penalty 1.66x at 1024 x 16
P x 4    ( 4224.0)  held-out  separated cells  28/29   worst measured penalty 1.20x at 4096 x 128
```

The three held-out worst penalties — **1.81x, 3.04x, 1.20x** — are exactly the ones `perturbations.txt` reports for the complete three-parameter model, and the fitted worst cell is 1,024 rows of 128 contributors, which is the cell the spike README already names. The agreed/total counts differ from that file's because this script's separation rule pairs the fold against the *best parallel* strategy, which is the binary decision the compiler makes, while `fit.rs` also reports the three-way winner. The scaling to integers is exact: `max(work/P, depth)` needs a division and `max(work, depth*P)` does not.

### Required evidence, item by item

- **Mutation-proved on the term.** `perturbing_the_declared_cost_row_moves_the_selected_reduction` names 1,024 rows of 64 contributors — just under the fitted 1,056, where the contour runs, because the selector's crossing is at `rows ~ P`. At the fitted value it selects the serial fold; at a quarter it still does; **at four times it selects a parallel plan.** The perturbation scale is the sweep's own, which matters because the row is determined only to about a factor of four. The same test drives one row of 4,096 contributors at all three values and finds the verdict unmoved, so the flip is a contour rather than a global switch.
- **Unchanged golden for a silent profile.** `a_profile_declaring_no_cost_row_selects_and_encodes_exactly_as_before` asserts both halves at the shape the row does move: the canonical descriptor is byte-identical to a profile built without the family, the row resolves `Unknown`, and the structural winner is still selected. The descriptor half is the stronger one, and it is a property of the encoding: the cost-row section is written **only when non-empty**, following the evaluation-order precedent, so `COMPLETE_PROFILE_DESCRIPTOR_DOMAIN` stays at `v11` and no existing profile's identity moves. `the_declared_profile_states_the_measured_cost_row` drives the same in `tiler-build` by removing the row from `FIRST_MACOS_APPLE9` and observing exactly 100 bytes come back off.
- **Explain names the term and both sides of the `max`.** Verbatim, from the compile at one row of 4,096 contributors:

  ```text
  198 costing higher-cost rule=tiler.cost.measured-fold-steps.v1@1 provider=tiler.compiler@1 subject=alternative:program-alternative:4cb76c75ed746087 event=cost:tiler.cost.measured-fold-steps.v1:assumption:higher-cost:fold-steps:operations=4329472,saturated-parallel-fold-steps:operations=1056,span-steps:operations=4326432,work-steps:operations=8192 causes=197
  200 costing higher-cost rule=tiler.cost.measured-fold-steps.v1@1 provider=tiler.compiler@1 subject=alternative:program-alternative:f2116fb1acb5981f event=cost:tiler.cost.measured-fold-steps.v1:assumption:higher-cost:fold-steps:operations=291328,saturated-parallel-fold-steps:operations=1056,span-steps:operations=288288,work-steps:operations=8448 causes=199
  202 costing retained rule=tiler.cost.measured-fold-steps.v1@1 provider=tiler.compiler@1 subject=alternative:program-alternative:1b00ae52b28b9872 event=cost:tiler.cost.measured-fold-steps.v1:assumption:retained:fold-steps:operations=139264,saturated-parallel-fold-steps:operations=1056,span-steps:operations=136224,work-steps:operations=8256 causes=201
  203 selection selected rule=tiler.selection.structural-pareto.v1@1 provider=tiler.compiler@1 subject=alternative:program-alternative:1b00ae52b28b9872 event=selection:tiler.selection.structural-pareto.v1:selected causes=202
  ```

  Record 198 is the serial fold, 200 the tree, 202 the selected split. A reader can see **which side of the `max` decided**: every plan's span term dwarfs its work term, so this shape is span-bound, which is the model's own statement of why parallelizing pays where the row count cannot saturate the device. The selection row cites the measured record as its cause, so the reason is on the causal path from the verdict rather than beside it. The basis is `assumption` and not `checked-invariant`: the work and span counts are exact, but the row combining them is fitted.
- **Shapes checked against the retained TSV.** `the_selection_agrees_with_the_retained_sweep` takes two **separated** cells from the sweep's own results and asserts the selector agrees with both: 1,024 x 4,096, where the retained medians are 250.31 microseconds for the fold against 203.23 for the tree and 207.66 for the split, so parallelizing is measured faster; and 16,384 x 32, where they are 27.57 against 50.16 and 31.91, so the fold is. Neither is re-argued.
- **Hard feasibility untouched.** The measured term ranges only over plans the frontier admitted and the boundary reconciliation composed, so no infeasible plan is in the set to prefer. Nothing in `target/feasibility.rs`, the frontier's admission, or the boundary reconciliation changed. A cost row resolves `Declared`, `Deferred`, or `Unknown`, and the last two are treated identically as *no preference* — never as a refusal, never as a zero.

### Identity: the pinned population, enumerated with before and after

**These values were computed on this branch from base `fe3ad943` and must be recomputed on the merged tree.** Four pins moved and no others; nothing else in the workspace changed identity, which the 2,966-test workspace run is the evidence for.

| pin | before | after |
| --- | --- | --- |
| canonical descriptor length (`metal_declaration.rs`, `the_declared_profile_states_one_barrier_realization`) | 1,999 | **2,099** |
| standard Metal artifact identity (`metal_plan.rs`) | `23c46a19f6bc601d35bf4ca653e890372da3079b1bb60526220dc3b3221dcdd0` | **`357f06767e459ea99fb45a11d6aaffd01f46051a941ec2f1e3eed54ae4290b73`** |
| standard Metal cache subject (`metal_plan.rs`) | `e89c4d826149c9d103e2ed8392968c0c519df454e23e7793932bc33bc86b1595` | **`c626e43b6cfc64ccb828f0394c0a641e0d01d7f54bcb3b506cdc3b8651dac59b`** |
| published envelope fixed content (`metal_plan.rs`) | 64,542 | **65,242** |

The authority-ledger paragraph that mirrors those three values was moved with them.

**The delta is encoding-predicted to the byte rather than observed.** The cost-row section writes a length-prefixed 33-byte domain separator (41), a row count (8), a length-prefixed 34-byte row key (42), a fixed-width `u64` (8), and a one-byte compact source index: **100 bytes exactly**, with no source-table growth because the row shares the measured source the grid-axis, dispatchability, and numerical rows already carry. The envelope embeds that descriptor seven times, so the fixed content grew by exactly 700. **That the arithmetic closes is the evidence no layout moved.**

### The public surface added, and its labelling

Four items, all **reviewed draft boundaries** under ADR 0074 convention 7 and ADR 0075, labelled in `crates/tiler-compiler/src/target.rs`'s module header in the same paragraph shape the accepted evaluation-order family uses, saying expressly that the acceptance covers the model and not the spelling:

```rust
pub enum TargetCostRowResolution { Declared { value: u64 }, Deferred { available_at: AvailabilityPhase }, Unknown }
impl TargetProfileBuilder {
    pub fn declare_saturated_parallel_fold_steps(&mut self, steps: u64, source: TargetFactSource) -> Result<(), TargetProfileBuildError>;
    pub fn declare_measured_saturated_parallel_fold_steps(&mut self, steps: u64, source: TargetCompileProfileMeasurementSource) -> Result<(), TargetProfileBuildError>;
}
impl TargetProfile {
    pub fn saturated_parallel_fold_steps(&self, available_phase: AvailabilityPhase) -> TargetCostRowResolution;
}
// plus one variant on the existing non_exhaustive error enum:
TargetProfileBuildError::DuplicateCostRow { row: &'static str, phase: AvailabilityPhase }
```

The row enum itself is **private**: the public surface is one pair plus one reader per row, exactly as the quantitative axes are spelled, so a second row lands additively without widening the reviewed surface. An acceptance node should be filed against this exact list.

### Measurement boundaries carried forward

- **The retained sweep dispatched the tree at `governed_partition`'s balanced split**, because `MEASURED_TREE_PARTICIPANT_CAP` landed after it (`39702d21` after `54d2c9e6`). The compiler now emits the capped width, so at some shapes the tree it dispatches is not the tree that was timed. That moves *which parallel plan* is preferred and not *whether it parallelizes* — the distinction the sweep itself found consequential, since the two parallel strategies are inside each other's noise almost everywhere. Recorded in the ledger, the spike README, and the agreement test.
- **`P` is determined to about a factor of four**, so the contour's position is bounded rather than pinned. The mutation test drives exactly that band.
- **The activated model is a selector and not a latency estimate**, and nothing converts it to seconds.
- **`docs/research/embedding/self-contained-embedding.md` quotes the 1,999-byte descriptor** in a dated measurement paragraph. It is historically correct at its own commit and sits in `research/embedding`, which this ticket does not declare. Flagged rather than edited.

### What is not done, and why

- **The tree's `TensorRole::Output` hard-code in `single_workgroup_tree_region` is still unexercised.** Preference was the missing half and it landed; what is still needed is a *cover* that assigns the tree a materialized write, which is an epilogue-chain question this ticket does not own. The note there was updated to say so rather than left claiming preference is missing.
- **The split's partition rule is unchanged.** `governed_partition`'s doc says its improvement needs the same saturation quantity, and that is now declared — but choosing a width *within* a strategy is a second consumer of the number and belongs to `calibrate-device-cost-models`. The note there was updated to draw that line rather than silently absorb the work.

### Checks

All from the worktree, on the finished tree.

```text
cargo fmt                                                       exit 0
cargo check --workspace --all-targets                           exit 0
cargo nextest run -p tiler-compiler -p tiler-build              exit 0   841 passed
cargo clippy -p tiler-compiler -p tiler-build --all-targets -- -D warnings   exit 0
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps                  exit 0
cargo nextest run --workspace                                   exit 0   2,966 passed, 7 skipped
cargo test --workspace --doc                                    exit 0
python3 spikes/.../activated_selector_check.py                  exit 0
```

## Integrated 2026-08-07

Merged at `09b0d0b8`; the composed tree gates green with `make full` exit 0 — 2,966 workspace tests and 1,044 release numerical.

**The four identity pins were recomputed by the coordinator on the merged tree**, not carried from the branch. They came out identical to the branch's values, which is itself the evidence that nothing landing between this branch's base and the merge moved them: the realization witness vocabulary and the conformance crate admission both claimed no identity movement, and this confirms it. Descriptor length 1,999 → 2,099; artifact identity `23c46a19…` → `357f0676…`; cache subject `e89c4d82…` → `c626e43b…`; fixed content 64,542 → 65,242.

**The design question the acceptance left open was settled by measurement rather than by argument**, which is the outcome Tom's no-scope-cutting instruction was protecting. The cheap shape — a measured term breaking ties *inside* the non-dominated set — cannot express the retained measurement at all: the serial fold structurally dominates both parallel strategies, so `non_dominated()` holds exactly one plan on this family, and a tie-break inside a singleton decides nothing. `the_parallel_reduction_plans_are_structurally_dominated` asserts that rather than assuming it. The measured term therefore ranges over the retained *valid* plans and can prefer a structurally dominated one; it can never prefer an infeasible one, because none is in the set.

**Both reserved constraints were answered structurally rather than argued around.** No second cost-model key is minted — `PlanStructuralCost`, `dominates`, `aggregate_cost`, the frontier's single-key check and `non_dominated` are untouched, and `measured_cost` has no `dominates` — so Pareto pruning cannot go dark, because nothing new enters the relation. `PlanStructuralCost` was not widened with a latency dimension.

**One test observation, recorded per `AGENTS.md`.** The workspace run reported a single `leaky` verdict, on `tiler-compiler governed::contraction_conformance::the_four_prefill_cells_are_refused_by_the_unstaged_fold_and_reached_by_the_staged_one`. An earlier run today reported one on an unrelated `tiler-macros` test. A leaky verdict that **moves between unrelated tests** is the known macOS pipe-inheritance race rather than a real unreaped child, which is the distinction `AGENTS.md` draws; recurrence in one test would mean the opposite.

~~**The public surface is parked, not landed as accepted:**~~ **Correction — 2026-08-10.** [`accept-the-measured-cost-row-public-surface`](accept-the-measured-cost-row-public-surface.md) is `status: done`; Tom accepted the exact `declare_*` spelling on 2026-08-07. The Integrated sentence that called the surface parked was the landing-time split under ADR 0075 and is left as history. Residual bookkeeping — draft labels still on the accepted surface in `target.rs` / ledger — is owned by open [`retire-the-draft-label-on-the-accepted-cost-row-surface`](retire-the-draft-label-on-the-accepted-cost-row-surface.md) (`todo`), not by this selection-activation ticket.

**A measurement boundary that must survive this ticket.** The sweep dispatched the tree at the *balanced* split, and `MEASURED_TREE_PARTICIPANT_CAP` landed after it. That moves which parallel plan is preferred, not whether the program parallelizes, so the contour this row turns on is unaffected — recorded in the ledger, the spike README, and the test rather than only here.

**Flagged and deliberately not edited:** `docs/research/embedding/self-contained-embedding.md:67` quotes the old 1,999-byte descriptor inside a dated measurement paragraph. Historically correct at its commit, in an undeclared scope.

**Correction — 2026-08-10 (identity after-table).** The four pin after-values enumerated in Outcome and restated above (descriptor 2,099; artifact `357f0676…`; cache `c626e43b…`; fixed content 65,242) are the merge-time recomputation for this ticket's cost-row section. Later unrelated encoding steps moved standard Metal artifact/cache/fixed-content pins past that after-table; `metal_plan` documents the history and is the live test authority. Descriptor length for the qualified declaration remains 2,099 at this audit.
