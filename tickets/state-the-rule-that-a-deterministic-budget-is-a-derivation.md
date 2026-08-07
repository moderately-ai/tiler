---
id: state-the-rule-that-a-deterministic-budget-is-a-derivation
title: State the rule that a deterministic budget is a derivation
status: todo
priority: p3
dependencies: []
related: [derive-the-region-shape-budgets-from-the-declaration]
scopes: [implementation/compiler]
shared_scopes: [project/tickets]
paths: []
tags: [budgets, conventions]
---
## The rule to write down

**A deterministic budget is a formula over the declaration, or it carries a stated reason why it is not.** That rule is currently followed by five of the fourteen bounds in `DeterministicBudgets::governed` and merely exemplified rather than stated, which is how the three region-shape bounds drifted into being bare constants — the drift [`size-the-region-shape-budgets-to-the-programs-the-profile-admits`](size-the-region-shape-budgets-to-the-programs-the-profile-admits.md) had to correct on 2026-08-07.

Write it at `DeterministicBudgets`, where the next person adding a bound will read it.

## Why the rule, and not just the three fixes

**A constant sized to today's largest known program is a ceiling somebody has to raise.** Every budget is written into the canonical request subject, so each raise moves every artifact identity in the workspace. A bound that is a *derivation* tracks the declaration and never needs raising; a bound that is a *constant* structurally disincentivizes growing the supported envelope, because growing it costs an identity migration. That is the self-constraint the 2026-08-07 decision rejected, and the rule is what stops it recurring one field at a time.

**This has now happened twice.** `regions` was a constant and was corrected to a derivation over the declared outputs; the three region-shape bounds were constants and are being corrected the same way. Two instances of one pattern is the point at which the rule gets written rather than the third fix applied.

## Audit the remaining bounds against it

This ticket is not only documentation. Check each of the fourteen and classify it:

- **Derived already** — the five program-scoped bounds. Confirm each still matches its stated formula rather than assuming it; one of them was found stale in a different document this week.
- **Constant with a stated reason** — a bound whose value is a genuine determinism limit rather than a program-shape ceiling may legitimately be a constant. `region_expansions` (10,000), `region_covers` (1,024), `region_cover_expansions` (100,000) and `physical_plan_combinations` (4,096) are the candidates: they bound *search*, and exhausting one costs an alternative while coverage survives. If that is the reason, state it at each rather than leaving it inferable from the numbers' magnitude.
- **Constant with no stated reason** — the defect. `normalization_rewrites` is `8` and its doc explains what it bounds but not why eight. Either derive it or state the reason.

Report the classification for all fourteen. **A bound you cannot classify is a finding**, not something to leave in whichever bucket is convenient.

## Explicit non-goals

Do not change any value — [`derive-the-region-shape-budgets-from-the-declaration`](derive-the-region-shape-budgets-from-the-declaration.md) owns the three, and any other value change found necessary here is its own ticket with its own identity accounting. **No identity may move under this ticket**; if a value needs to change, stop and file it.

Do not pre-empt [`decide-whether-a-derived-budget-belongs-in-the-request-subject`](decide-whether-a-derived-budget-belongs-in-the-request-subject.md), which asks the sharper question underneath this one.

## Closes when

The rule is stated at `DeterministicBudgets`, all fourteen bounds are classified with the unclassifiable ones reported, every constant that stays a constant carries its reason at its own definition, and no value moved.
