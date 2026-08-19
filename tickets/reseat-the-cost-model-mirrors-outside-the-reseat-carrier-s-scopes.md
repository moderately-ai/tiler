---
id: reseat-the-cost-model-mirrors-outside-the-reseat-carrier-s-scopes
title: Reseat the cost-model mirrors outside the reseat carrier's scopes
status: todo
priority: p2
dependencies: []
related: [reseat-the-grid-and-cost-profile-rows-on-the-re-measured-records]
scopes: [research/program-planning, research/cost-model, contracts/optimizer, implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [metal, target-profiles, provenance, documentation]
---
## User-visible outcome

Every document and doc comment that states the saturated-parallel-fold-steps model's fitted value or accuracy either names the 2026-08-18 record that now sources the profile row, or is dated to the 2026-08-07 record it actually describes. No site states a superseded figure in the present tense, and no site tells a reader that the activated selector rests on a record the profile no longer cites.

## Why this exists

[`reseat-the-grid-and-cost-profile-rows-on-the-re-measured-records`](reseat-the-grid-and-cost-profile-rows-on-the-re-measured-records.md) reseated the authority ledger's grid-axis and saturated-cost rows onto the 2026-08-18 `26A5406e` records and repaired the mirrors inside its own scopes (`research/target-profiles`, `contracts/artifacts`, `implementation/build`). The sites below are outside those scopes and were left alone deliberately rather than reached across a fence; `implementation/compiler` was additionally held by another lane at the time.

## Facts, each read at `fad2d7d007a71b42e2c221d88313597a2803fc82` on 2026-08-19

Re-audit each at your own base before editing; these are stale the moment they are written.

1. **Fact — `crates/tiler-compiler/src/pipeline/tests.rs` carries a doc comment that contradicts the constant directly below it.** The comment reads `crates/tiler-compiler/src/pipeline/tests.rs "The fitted saturated-fold-step row of the retained 2026-08-07 sweep"` and `crates/tiler-compiler/src/pipeline/tests.rs "parallel_threads = 1.056e3"`, while the constant is `crates/tiler-compiler/src/pipeline/tests.rs "const MEASURED_SATURATED_FOLD_STEPS: u64 = 1_280"`. The value moved at the compilation-selection carrier's integration; the comment did not. This is the highest-signal site here, because the two sit five lines apart.
2. **Fact — `crates/tiler-compiler/src/measured_cost.rs`'s module doc attributes the activated model to the superseded record.** It reads `crates/tiler-compiler/src/measured_cost.rs "results/2026-08-07-apple-m4-max-macos27.0-26A5388g/"` and quotes that record's mutation figures: `crates/tiler-compiler/src/measured_cost.rs "26 separated cells to 20 and the worst penalty from 1.81x to 3.04x"`. The 2026-08-18 record's corresponding figures are 22 separated held-out cells, 20 agreed, worst regret 2.0106, dropping to 17 agreed and 3.6761 at a quarter — read them from `spikes/program-planning/reduction-dispatch-crossover/results/2026-08-18-apple-m4-max-macos27.0-26A5406e/calibration.txt` and `.../perturbations.txt` rather than from this ticket. The module's *argument* — that `step` is provably order-preserving and `encoder` is measured-inert — is reconfirmed by the new record (scaling `encoder` by twenty or `step` by a tenth moves the predicted winner at zero of the 92 cells), so what needs repair is the record cited and the figures, not the reasoning.
3. **Fact — `spikes/program-planning/reduction-dispatch-crossover/README.md` is written entirely from the 2026-08-07 session and does not mention the 2026-08-18 one.** Its result paragraph, its fitted-parameter table (`spikes/program-planning/reduction-dispatch-crossover/README.md "1.056e3 | fold steps retired at once when saturated"`), its accuracy table, its mutation table, and its activated-selector section all state the superseded fit in the present tense. The 2026-08-18 records, `RUN.md`, and `smoke.txt` are committed in that directory with nothing in the README pointing at them. This is the largest piece of work in this ticket.
4. **Fact — the two sessions' tree cells are not like-for-like, so a rewrite may not simply swap numbers.** The 2026-08-07 sweep dispatched the tree at `governed_partition`'s balanced split; the 2026-08-18 harness reimplements `capped_tree_partition`'s nearest-admissible-to-256 rule and checks it against every published launch. At one row the tree width is 8 against 4 at sixteen contributors, 32 against 8 at sixty-four, and 256 against 32 at 1,024. The comparable quantity across the two sessions is the serial-versus-parallel verdict, never a raw timing or a speedup ratio.
5. **Fact — two further documents state the superseded accuracy figures in the present tense.** `docs/compiler/fusion-and-scheduling.md "24 of the 26 held-out cells whose verdict is separated from the noise"` and `docs/research/program-planning/flash-class-capability-set.md "reproduces the verdict on 24 of the 26 separated held-out cells"`. A third quotes the mutation figures as an argument about why a per-shape table would encode noise: `docs/research/cost-model/measured-feedback-tuning-loop.md "the held-out worst penalty from 1.81x to 1.20x"` — that argument survives on the new record (quadrupling `P` still improves held-out separated agreement, to 21 of 22 at a worst regret of 1.26x), so it needs re-sourcing rather than withdrawal.

## A question this ticket must answer rather than assume

`spikes/program-planning/reduction-partition-calibration` carries 1,056 as a *frozen protocol constant* in several places — `spikes/program-planning/reduction-partition-calibration/src/shape_aware.rs "const SATURATED_FOLD_STEPS: u64 = 1_056"`, the `INTERACTION_ROWS` matrices, and the row-regime feature `s=clamp(log2(rows/1,056),-1,1)` its README freezes. Those are that spike's own pre-registered parameters, frozen before timing by [`test-row-regime-divisor-interactions-on-a-fresh-tree-width-matrix`](test-row-regime-divisor-interactions-on-a-fresh-tree-width-matrix.md), and moving a frozen parameter after the fact would destroy the pre-registration that makes its result mean anything. The likely correct answer is that they stay and gain a note recording that the profile row has since moved to 1,280; do not change one without reading that ticket's freeze first.

## Closes when

Every site above either cites the 2026-08-18 record or is dated to the record it describes; no superseded figure is stated in the present tense; the tree-width non-comparability is stated wherever the two sessions' numbers appear together; the frozen-constant question is answered explicitly rather than by silence; and `make citations`, `tkt lint`, and the touched crates' gates are green.
