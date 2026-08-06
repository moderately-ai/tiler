---
id: correct-the-l6-budget-item-for-the-output-derived-budgets
title: Correct L6's budget item for the output-derived budgets
status: review
priority: p2
dependencies: []
related: [bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals, correct-the-l6-budget-refusal-item]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, program-planning, documentation]
claimed_from: todo
assignee: agent-l6-budgets
lease_expires_at: 1786034532
---
## User-visible outcome

L6's discharged-budget item quotes the five derivations and the five governed values as they were before the budgets became functions of the declared *output* arity, and the record states them as **Fact** with a source read. A reader taking any of the five numbers or any of the three derivations from it today is taking a stale fact from a record that says it read the source.

## Why this exists

**Fact — the sentence at `docs/research/program-planning/complete-model-ingestion-and-execution.md:210`** states, as a source-read fact, that `check_program_budgets` checks `regions` against the constant three, `host_expression_nodes` against `input_count() * 2 + 7`, and `buffers` against `input_count() + 3`, and that the widening landed `semantic_values` 80, `semantic_operations` 62, `host_expression_nodes` 43, `buffers` 21, `regions` unchanged at 3.

**Fact — four of those eight statements were already stale before this correction is owed** (`regions` moved from 3 to 4 with `admit-elementwise-epilogues-over-a-materialized-intermediate`), and [`bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals`](bound-the-assembled-region-count-and-derive-the-multi-output-budget-actuals.md) (worker commit `8a78f079`) moved the rest. The current source, read on that branch:

- `regions` against `output_count() * 4`, governed `12`
- `host_expression_nodes` against `input_count() * 2 + output_count() * 4 + 3`, governed `51`
- `buffers` against `input_count() + output_count() * 4`, governed `30`
- `semantic_values` `80` and `semantic_operations` `62`, both unchanged

**Fact — the record's own sizing basis is what forced the move.** The item's parenthetical, "a region count belongs to a plan and no layer plan exists to count", is the reasoning that made `regions` a constant, and it survives; what does not is its conclusion, because a plan covers every declared output and the layer declares three. The producing ticket's Outcome carries the derivation and the measurement behind the per-output four.

**Inference — item 1's verdict does not change.** P2 still exceeds the budgets it exceeded and the widening still discharges the refusal; what is wrong is the arithmetic a reader would reproduce from this item, which is exactly the failure mode `correct-the-l6-budget-refusal-item` already corrected once.

## Scope note

`docs/research/program-planning/**` is `research/program-planning`, which the producing compiler ticket does not hold and could not add without expanding its own outcome into the research corpus. Read from `ticketsplease.toml` rather than asserted. `docs/research/program-planning/model-level-qualification.md` should be swept in the same change for the same numbers.

## Closes when

The item's derivations and values match the source at the commit the correction is written against, the correction states which sentence moved and why, and a `grep -rn 'input_count().saturating_add(3)\|regions. against the constant' docs/` returns nothing.

## Outcome

**Source verified first, on this branch's base `1a2d8b26`, by reading `check_program_budgets` and `DeterministicBudgets::governed` in full rather than grepping.** The source agrees with the dispatch brief on all five: `regions` against `program.output_count() * 4` governed `12`; `host_expression_nodes` against `input_count() * 2 + output_count() * 4 + 3` governed `51`; `buffers` against `input_count() + output_count() * 4` governed `30`; `semantic_values` `80` and `semantic_operations` `62` unchanged. `check_budget` (`request.rs:5625`) refuses only on `actual > limit`, which is what makes P2's five actuals — 80, 62, 12, 51, 30 — a zero-margin pass rather than a comfortable one. No three-way disagreement arose; the record alone was stale.

**The two moves are dated and attributed from git rather than from the brief.** `62c63061` (2026-08-05) took the D-18 widening leaving `regions` at 3; `76f1cf96` (2026-08-06) moved `regions` 3 → 4 with the epilogue as a fourth dispatch — confirmed by `git show 76f1cf96 -- crates/tiler-compiler/src/request.rs`, which shows `regions: 3` → `regions: 4` and the ticket file `admit-elementwise-epilogues-over-a-materialized-intermediate.md` in the same commit; `8a78f079` (2026-08-06) replaced the constant with the per-output derivation and carried `host_expression_nodes` 43 → 51 and `buffers` 21 → 30. The moved pin is a **Fact** read at `crates/tiler-compiler/src/explain.rs:4183`, not relayed: `689c3aefc30f48d3` → `8966151e455093ea`, with the rebaseline comment naming this ticket's producer.

**The correction is tense-preserving.** Item 1's parenthetical ground — *a region count belongs to a plan and no layer plan exists to count* — is retained verbatim and shown to survive while its conclusion does not, on the source's own reasoning that a plan covers every declared output. The historical four-of-five count is retained as **Historical inference** with the note that neither of its expressions exists now, so a reader cannot reproduce it from today's derivations; against the same pre-widening values today's derivations put P2 over all five. **Item 1's verdict is unchanged in both directions** and the record says so: P2 exceeded what it exceeded, and the widening discharges it.

**Beyond the item, in the same change, three siblings of the same staleness.** The W-A row (line 62) carried the same superseded `buffers` derivation from the earlier correction — it now derives `input_count() + output_count() * 4` against a governed 30, with a 310-input program's actual at least 314, and its conclusion shown to survive every move. The record's closing two paragraphs asserted "five exact refusals still stand" and "two of the three programs would pass today's deterministic budgets … and the third would be refused by both"; both were falsified by the discharge the earlier correction recorded and neither was swept then. Four refusals stand, and all three programs now pass the budgets and are refused only by the recognizer (P1: 2 inputs, 1 output; P3: 3 inputs, 1 output — both far inside every bound).

**The L8 sweep found a counted population, not a single sentence.** `model-level-qualification.md` stated L6's refusal count in **seven** places, every one reading "five": lines 80, 110, 126, 188, 262, 280, 292. All seven now read four, and the correction note at line 80 names the population and carries the one command that reproduces it — `grep -c "L6's four standing refusals\|refuses at four places" docs/research/program-planning/model-level-qualification.md` returns `7`. Line 110 additionally carried the stale arithmetic proper ("P2 exceeds three of four") and now carries the governed values with the discharge attributed to both commits. A first draft of that note said "six"; the count was checked against the file rather than asserted, which is what caught it.

**Closes-when check.** `grep -rn 'input_count().saturating_add(3)\|regions. against the constant' docs/` returns nothing (exit 1). The first pass **failed** it: the W-A correction re-quoted the superseded spelling inside its own historical parenthetical, so the check fired on the correction rather than on the defect. The parenthetical was reworded to state the derivation in words, which is the reading a historical note needs anyway. Wider sweep for spellings the closes-when grep misses — `saturating_add(7)`, `input_count() * 2 + 7`, `4.max(input_count`, the governed pair 43/21, `regions` unchanged at 3, "three of four" — returns only the intended historical mentions inside dated corrections (`43 → 51`, `21 → 30`, and the quoted prior text).

**Out of scope, reported not edited.** `docs/roadmap.md:402` (scope `contracts/navigation`, per `ticketsplease.toml`) carries the L6 ladder row's "five exact refusals stand between the design and a compiled model" — the same staleness, in a scope this ticket does not hold.

**Checks.** Docs-only; no gate input touched — `git diff --name-only 1a2d8b26 HEAD` is exactly `docs/research/program-planning/complete-model-ingestion-and-execution.md`, `docs/research/program-planning/model-level-qualification.md`, and this ticket, none of them in the gate-input set, so the delta may ride the most recent green gate. `git diff --check` clean; `tkt lint` reports no problems.

**Guard verdict is WARN, not clean, and the WARN is vacuous — stated rather than rounded off.** `tkt guard --base 1a2d8b26 tkt/correct-the-l6-budget-item-for-the-output-derived-budgets` reports `changed files: 3`, `affected scopes: project/tickets, research/program-planning`, `declared scopes: project/tickets, research/program-planning` — no undeclared scope — then a long collision list. Every shared collision is `project/tickets`, which each claimed ticket declares. The **direct** collisions number six and all are on `research/program-planning`: `audit-backend-authoring-against-all-thirteen-responsibilities`, `correct-the-four-thread-grid-rationales-the-measured-row-falsified`, `decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode`, `govern-the-three-ungoverned-spike-records`, `measure-executable-coverage-identity-growth-against-the-program-identity-bound`, and `restore-the-spikes-against-the-composed-numerical-contract`. **All six are `done`, and each one's `tkt/<id>` branch is an ancestor of the base** — `git merge-base --is-ancestor tkt/<id> 1a2d8b26` succeeds for all six — so they are stale declarations on merged work, not live holders. The two genuinely live claims at the time of this commit, `admit-the-registered-elementary-families-as-recognizable-program-stages` and `compile-an-elementary-function-golden-through-the-metal-toolchain`, were read **branch-side** (`git show tkt/<id>:tickets/<id>.md`) rather than from the integration copy: their scopes are `implementation/ir, implementation/compiler` and `implementation/metal`, neither reaching `research/program-planning`. Both branches carry zero commits, so their file-level disjointness is *vacuous* rather than diff-verified; the non-vacuous evidence is the branch-side scope declarations.
