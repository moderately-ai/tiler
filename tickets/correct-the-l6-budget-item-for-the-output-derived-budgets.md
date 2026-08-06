---
id: correct-the-l6-budget-item-for-the-output-derived-budgets
title: Correct L6's budget item for the output-derived budgets
status: in-progress
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
