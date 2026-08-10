---
id: move-the-bf16-optimizer-legality-ledger-cell
title: Move the BF16 optimizer legality ledger cell
status: done
priority: p3
dependencies: [establish-bf16-optimizer-legality]
related: [establish-bf16-optimizer-legality]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, support-matrix]
---
## What this owes

`docs/dtype-support.md`'s BF16 **`Optimizer legality`** cell reads `absent/unsupported`. [`establish-bf16-optimizer-legality`](establish-bf16-optimizer-legality.md) landed on 2026-08-07 and is what moves it. That ticket could not: `contracts/navigation` was outside its scopes, and `AGENTS.md` requires that when work advances a support-matrix row, the row and its extent are named or the ledger update is filed — this is the filing.

## State the extent, not just the row

**The claim is narrower than "BF16 legality is established", and the cell must say so.** Every obligation was discharged as **Derived**, none as Measured. The four reduction obligations were discharged **vacuously** — the BF16 vocabulary is exactly three families (constant, multiply, add) with no reduction, no contraction-capable family and no coordinate relation — and were classed `SoundProof` over an empty population rather than `NormativeGuarantee`. **BF16 reassociation remains `Unknown` and is explicitly withheld** at the operation vocabulary.

So the cell moves for the vocabulary that exists, and a BF16 fold family registered later reopens the four vacuous obligations with a real population under them. `AGENTS.md` requires maturity and evidence claims stay distinct; a cell reading as though a fold had been proved legal would be exactly the overstatement it warns about.

## Check the neighbours before editing

Other BF16 cells moved the same day and may now be stale in either direction — the recognizer widening, the device-executed vertical, and the conformance-crate landings all touched what BF16 can reach. Read the whole BF16 row rather than the one cell, and say what you checked. `correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents` names this file for a different reason; coordinate rather than collide.

## Closes when

The cell states the derived legality with its extent and its vacuous half, the rest of the BF16 row is checked and either correct or corrected, and no cell claims more than its evidence.

## Outcome — delivered 2026-08-07 at `6430b9f5`

The cell moved from `absent/unsupported` to **"tested guarantee, derived for constant/multiply/add only; the reduction obligations discharge vacuously"**, with a dated paragraph carrying the extent rather than restating the legality argument.

**Three bounds are stated as maturity claims**, which is what this ticket existed for: every obligation is *derived*, and **none is discharged `Empirical`** — with `FusionEvidenceClass`'s distinctness named as the reason a derived row cannot be read as a measured one; the four reduction obligations discharge **vacuously**, classed `SoundProof` over an empty population, because the BF16 vocabulary is exactly constant/multiply/add with no reduction, contraction-capable family or coordinate relation registered at that width; and BF16 reassociation stays `Unknown`, withheld at the operation vocabulary. Finding 28 is recorded as *decided* — the profile's authority, resolved by subject before a region reaches the derivation.

**Correction — 2026-08-10.** Delivery text above equates withheld reassociation *permission* with evidence class `Unknown`. Live `docs/dtype-support.md` same-day refinement replaces that with "BF16 reassociation is not proved here, merely not required" because `push_reduction_obligations` discharges `ReductionReassociation` as `SoundProof` when the region has no reduction; `Unknown { "unproven-reassociation" }` needs a reduction *and* a permitting contract. Permission remains withheld at the operation vocabulary (`BF16_FACT_REASSOCIATION_PERMITTED` false). The obligation outcome for the vacuous pointwise case is not `Unknown`.

**It carries its own reopening condition**: registering a BF16 fold family reopens all four obligations with a real population, and the cell must then be re-derived rather than carried. Verified against `fusion_legality.rs` rather than against the ticket alone.

### Reading the whole row found the more valuable defects

**Two cells were *understated*, which is as wrong as overstating and much easier to miss.** `Backend execution` read `absent/unsupported` when BF16 had already executed on a device; `Conformance evidence` read "no end-to-end run" when a device run exists. Both landings that caused the drift had explicitly recorded that these cells "need a `contracts/navigation` holder" — so the obligation was filed and simply never picked up until a worker read the row rather than the cell.

`Backend lowering`'s qualifier "offline emission and compilation **only**" became false the moment the same emission was dispatched. And `ABI and materialization` was **kept word for word**, with the new paragraph saying *why* it survives: the device run never builds an artifact envelope. Keeping a cell and explaining why is the harder call than moving it.

Six stale current-state clauses in the family notes were repaired as appended dated corrections rather than rewrites, per the file's own convention, and the recipe's non-monotone worked example was rewritten because it claimed BF16 had no rung-8 legality and no dispatch.

### Four items reported rather than edited, all belonging to a sibling that could not batch

At delivery, [`correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents`](correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents.md) could not batch with this ticket on `contracts/navigation` and was left unclaimed; the four items below were reported into its brief rather than edited here. That sibling is now `status: done` and discharged them (Physical carrier widened; dtype-f32 / recognizer-era prose repaired; roadmap recheck updated). Historical handoff list:

1. Three `dtype-f32` occurrences in this file were untouched at this ticket's land, and the new paragraphs deliberately cited no test name and never restated the wall — the "does not cross" claim was written as a property of the run rather than of a rule that no longer existed.
2. **`Physical carrier`'s qualifier "schedule-assembled regions only" may have understated**, since the recognizer widening replaced the constant carrier with a contract-derived one and a single-occurrence BF16 program reaches a selected plan.
3. **That ticket's own body was partly stale** at this ticket's land — its "What is true now" said optimizer legality was unreachable, written before `establish-bf16-optimizer-legality` landed.
4. **`docs/roadmap.md`'s reduced-precision-float row** carried the same staleness at greater length; its recheck still said the recognizer "refuses a pure-BF16 program with `dtype-f32`".

**Delta rule confirmed by the coordinator against the merge's own file list**: one file, `docs/dtype-support.md`, touching none of the build-configuration set, so it carries the latest green gate with `tkt lint` rerun.
