---
id: retire-the-draft-label-on-the-accepted-cost-row-surface
title: Retire the draft label on the accepted cost row surface
status: done
priority: p3
dependencies: []
related: [accept-the-measured-cost-row-public-surface]
scopes: [implementation/compiler, contracts/optimizer]
shared_scopes: [project/tickets, research/embedding]
paths: []
tags: [docs, public-boundary]
---
## What this owes

**Two records the cost-row landing left behind, both stating something no longer true.**

**1. The draft label.** `crates/tiler-compiler/src/target.rs`'s header labels the measured cost row's surface a draft, saying the acceptance covered the model and not the spelling. **Tom accepted the spelling on 2026-08-07** under [`accept-the-measured-cost-row-public-surface`](accept-the-measured-cost-row-public-surface.md), so the marker is now a stale disclosure — the exact class of drift this repository keeps finding. The header covers all five accepted items. Four items also carry their own `Draft public surface` block: `TargetCostRowResolution`, `declare_saturated_parallel_fold_steps`, `declare_measured_saturated_parallel_fold_steps`, and `TargetProfile::saturated_parallel_fold_steps`; `TargetProfileBuildError::DuplicateCostRow` has no separate marker. Retire the complete header-plus-four population without inventing a fifth per-item block.

**Retirement form — mirror the evaluation-order house pattern, do not delete-only.** Rewrite the measured-cost-row header and the four per-item markers to the same **`Accepted public surface`** shape already used for evaluation-order in `target.rs`: name [`accept-the-measured-cost-row-public-surface`](accept-the-measured-cost-row-public-surface.md) and the 2026-08-07 acceptance (and the five accepted items on the header). Mechanical deletion of the word "draft" that leaves "carries *no* acceptance of its own" or "expressly excluded the exact spelling" would still be a false live claim. Keep the surrounding load-bearing rationale — why the measured constructor carries a `TargetCompileProfileMeasurementSource`, why a cost row is not a capability axis, and why silence resolves `Unknown` / means *no preference* rather than making a profile unexecutable. Do not invent a fifth per-item block on `DuplicateCostRow`.

**The live compiler contracts under `contracts/optimizer` moved too.** That scope is every file under `docs/compiler/**` (`ticketsplease.toml`). Two live sentences still call the accepted declaration-pair spelling a reviewed draft boundary:

- `docs/compiler/optimizer.md`, source anchor `the exact public spelling of the declaration pair remains a reviewed draft boundary`
- `docs/compiler/cost-model.md`, source anchor `declaration pair is a reviewed draft boundary under ADR 0075`

Correct both in the same carrier; otherwise source and durable contracts remain contradictory after the labels disappear. Closing after an optimizer-only edit would leave a second live contract still calling the spelling a draft.

**2. A dated measurement quoting a descriptor length that moved.** `docs/research/embedding/self-contained-embedding.md`, source anchor `with a 1,999-byte canonical descriptor`, still quotes the canonical descriptor at **1,999 bytes**. The cost row's section moved it to **2,099**. The paragraph is a dated measurement and was correct at its commit, so the repair is **not** to overwrite the number: date the 1,999 observation and state the cost-row step to 2,099, the way the repository's other superseded measurements are handled. A reader reconciling an older record needs to see both values and the step between them. Do **not** overwrite with a live fixed-content total that later envelope steps have already moved again.

That length feeds the envelope seven times over, which is why the cost row's section moved the fixed content 64,542 → 65,242 — **and it has moved again since**, to 65,294 under the index-region `v10 → v11` step and further under later work. Read the live figure from `crates/tiler-build/src/metal_plan.rs` rather than quoting any superseded total from this ticket; the point of the repair is that the record carries the *descriptor-length ladder* for this subject, so add the 1,999 → 2,099 step without asserting a fixed-content total that a later step will falsify again.

If the document draws any conclusion from the 1,999 figure rather than merely reporting it, say whether the conclusion survives the new value — that is the part a number swap would silently break. (The neighbouring metallib byte-identical attribution does not depend on the absolute descriptor length.)

## Explicitly not in scope

No behaviour change, no signature change, no identity movement.

**Read the current pinned values from `crates/tiler-build/src/metal_plan.rs` at your own base and hold *those* still — do not take them from this ticket.** This paragraph originally named `357f0676…` / `c626e43b…` / 65,242, which were current when it was filed on 2026-08-07 and were superseded hours later by the index-region `v10 → v11` step under [`bound-a-symbolic-index-coefficient-interval-from-its-declared-extent`](bound-a-symbolic-index-coefficient-interval-from-its-declared-extent.md). A worker holding this ticket's literal values would have stopped on a difference that was correct.

The rule is what matters and it does not go stale: **this ticket moves no pin.** Record whatever the three read at your base, and if any differs after your change, stop — the change is wider than this ticket describes.

Do not touch the measurement boundary the acceptance preserved: the sweep dispatched the tree at the balanced split, and `MEASURED_TREE_PARTICIPANT_CAP` landed after it. That bound stands and this ticket does not widen it.

Out of declared scopes (not a close condition here): `docs/research/target-profiles/first-macos-metal-compile-profile-authority-ledger.md` still has a Cost rows paragraph denying spelling acceptance and a "What those pins are today" triple behind live `metal_plan.rs` pins. That file sits under `research/target-profiles`, not this ticket's scopes. The done ticket [`activate-measured-reduction-selection-from-a-target-cost-row`](activate-measured-reduction-selection-from-a-target-cost-row.md) still has historical Outcome text that the surface is parked — not a live close condition for this retirement.

## Closes when

No draft marker remains in the header-plus-four source population (header and four items rewritten to the evaluation-order **Accepted public surface** house form naming the acceptance ticket and 2026-08-07 date; no leftover "no acceptance" / "expressly excluded" prose on those sites), every live `docs/compiler/**` sentence that called the cost-row declaration-pair spelling a draft is corrected — at filing-time bases that is both `docs/compiler/optimizer.md` and `docs/compiler/cost-model.md` — the embedding record keeps 1,999 as a dated measurement and carries the cost-row step to 2,099, `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p tiler-compiler` passes, and no pin moved.

Suggested check after source/contract edits: `rg -n "Draft public surface|reviewed draft boundary" crates/tiler-compiler/src/target.rs docs/compiler/optimizer.md docs/compiler/cost-model.md` should show no remaining cost-row draft claim on the accepted spelling (unrelated drafts such as ScalarArithmetic may remain).

## Graph maintenance

Filed 2026-08-07 by the coordinator at acceptance. `research/embedding` is declared shared because the second half touches a research record the cost-row worker flagged and deliberately did not edit from its own scopes.

## Fact audit — 2026-08-10

Phase B ticket-only repair from the 2026-08-10 documentation audit. Product edits (source, contracts, embedding) are still owed; this section only fixes the ticket record.

1. **[VERIFIED]** Draft labels still live on the measured-cost-row header and four `Draft public surface` item blocks in `crates/tiler-compiler/src/target.rs`; spelling acceptance is recorded `done` under [`accept-the-measured-cost-row-public-surface`](accept-the-measured-cost-row-public-surface.md).
2. **[FALSE as complete contract population]** Naming only `optimizer.md` was incomplete: under the same `contracts/optimizer` scope, `docs/compiler/cost-model.md` still states `declaration pair is a reviewed draft boundary under ADR 0075`. What this owes / Closes when now name both files.
3. **[IMPRECISE]** "Remove the marker only" could be read as delete-only surgery that leaves false "no acceptance" / "expressly excluded" header prose. Retirement form is now the evaluation-order **Accepted public surface** rewrite, preserving capability-vs-cost and measurement-source rationale.
4. **[VERIFIED]** Embedding still quotes `with a 1,999-byte canonical descriptor`; repair remains dated supersession plus the 1,999 → 2,099 cost-row step, not a silent overwrite or a live fixed-content total.
5. **[OUT OF SCOPE residual]** Authority ledger Cost rows / "pins today" drift under `research/target-profiles`; historical "surface is parked" Outcome on the activate ticket. Neither is a close condition of this ticket.

## Outcome — 2026-08-10

The product edit was made from exact base `deae8305471e2e2f944bcdbd0175fdfd458134b9`. The complete source, both compiler contracts, embedding record, acceptance ticket, live pin test, and measurement-authority boundary were re-read before editing. The five-item accepted surface is unchanged: the declaration methods remain one paired item, and no fifth per-item marker was invented for `TargetProfileBuildError::DuplicateCostRow`.

The measured-cost-row header and its four existing item blocks now use the repository's **Accepted public surface** form and name Tom's 2026-08-07 acceptance under `accept-the-measured-cost-row-public-surface`. The optimizer and cost-model contracts now state that same acceptance. The capability-vs-cost distinction, `TargetCompileProfileMeasurementSource`, silent-row `Unknown` / no-preference behavior, and balanced-split evidence boundary remain intact.

The embedding record preserves the 1,999-byte descriptor as a dated 2026-08-06 Measurement and records only the measured-cost-row step to 2,099 bytes. It makes no current fixed-content claim and explicitly preserves the byte-identical `metallib` attribution. At the exact base the live pins are artifact `39e765637a7e014adac2b8a30788798758ca46584b558732c2bda41b7639ddda`, cache `7e00d9fa0ce90749e6f7d3d42e0f2aaabe5670e0359a0c20d1580a09bb967130`, and fixed content `65_313`; `crates/tiler-build/src/metal_plan.rs` is untouched.

The final residual census over `target.rs`, `optimizer.md`, and `cost-model.md` has no `Draft public surface`, `reviewed draft boundary`, `carries *no* acceptance`, or `expressly excluded the exact spelling` hit for this family. A subject perturbation changed the `TargetCostRowResolution` marker back to `Draft public surface`; the same census found `crates/tiler-compiler/src/target.rs:1767`, and restoration returned it to empty. This proves the residual check reaches a real accepted-surface site without treating a prose grep as semantic authority.

### Independent-review correction — 2026-08-10

Review of exact commit `5d133b08c17e01ef9a8a6e107a1939613c801fdd` found that the module header's following sentence, `Four exclusions were accepted with it`, acquired the measured-cost-row paragraph as its nearest antecedent. Those exclusions belong to the separately accepted evaluation-order-preservation family. The sentence now names that family explicitly, preserving both accepted surfaces' provenance without changing either surface.
