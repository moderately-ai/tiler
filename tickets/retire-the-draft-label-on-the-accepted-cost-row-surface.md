---
id: retire-the-draft-label-on-the-accepted-cost-row-surface
title: Retire the draft label on the accepted cost row surface
status: todo
priority: p3
dependencies: []
related: [accept-the-measured-cost-row-public-surface]
scopes: [implementation/compiler]
shared_scopes: [project/tickets, research/embedding]
paths: []
tags: [docs, public-boundary]
---
## What this owes

**Two records the cost-row landing left behind, both stating something no longer true.**

**1. The draft label.** `crates/tiler-compiler/src/target.rs`'s header labels the measured cost row's surface a draft, saying the acceptance covered the model and not the spelling. **Tom accepted the spelling on 2026-08-07** under [`accept-the-measured-cost-row-public-surface`](accept-the-measured-cost-row-public-surface.md), so the marker is now a stale disclosure — the exact class of drift this repository keeps finding. Retire it for `TargetCostRowResolution`, `declare_saturated_parallel_fold_steps`, `declare_measured_saturated_parallel_fold_steps`, `TargetProfile::saturated_parallel_fold_steps`, and `TargetProfileBuildError::DuplicateCostRow`.

Remove the marker only. **Do not reword the surrounding rationale** — why the measured constructor carries a `TargetCompileProfileMeasurementSource`, and why silence resolves `Unknown` rather than making a profile unexecutable, both survive their acceptance and are the reasons the shape is what it is.

**2. A dated measurement quoting a descriptor length that moved.** `docs/research/embedding/self-contained-embedding.md:67` quotes the canonical descriptor at **1,999 bytes**. The cost row's section moved it to **2,099**. The paragraph is a dated measurement and was correct at its commit, so the repair is **not** to overwrite the number: date it and state what moved it, the way the repository's other superseded measurements are handled. A reader reconciling an older record needs to see both values and the step between them.

That length feeds the envelope seven times over, which is why the cost row's section moved the fixed content 64,542 → 65,242 — **and it has moved again since**, to 65,294, under the index-region `v10 → v11` step. Read the live figure rather than quoting either of those; the point of the repair is that the record carries the *ladder*, so add the step this ticket is about without asserting a total that a later step will falsify again.

If the document draws any conclusion from the 1,999 figure rather than merely reporting it, say whether the conclusion survives the new value — that is the part a number swap would silently break.

## Explicitly not in scope

No behaviour change, no signature change, no identity movement.

**Read the current pinned values from `crates/tiler-build/src/metal_plan.rs` at your own base and hold *those* still — do not take them from this ticket.** This paragraph originally named `357f0676…` / `c626e43b…` / 65,242, which were current when it was filed on 2026-08-07 and were superseded hours later by the index-region `v10 → v11` step under [`bound-a-symbolic-index-coefficient-interval-from-its-declared-extent`](bound-a-symbolic-index-coefficient-interval-from-its-declared-extent.md). A worker holding this ticket's literal values would have stopped on a difference that was correct.

The rule is what matters and it does not go stale: **this ticket moves no pin.** Record whatever the three read at your base, and if any differs after your change, stop — the change is wider than this ticket describes.

Do not touch the measurement boundary the acceptance preserved: the sweep dispatched the tree at the balanced split, and `MEASURED_TREE_PARTICIPANT_CAP` landed after it. That bound stands and this ticket does not widen it.

## Closes when

No draft marker remains on the five accepted items, the embedding record carries both descriptor values with the step between them, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-compiler` passes, and no pin moved.

## Graph maintenance

Filed 2026-08-07 by the coordinator at acceptance. `research/embedding` is declared shared because the second half touches a research record the cost-row worker flagged and deliberately did not edit from its own scopes.
