---
id: widen-the-identity-growth-ladder-to-the-governed-operation-budget
title: Widen the identity-growth ladder to the governed operation budget
status: in-progress
priority: p2
dependencies: []
related: [measure-executable-coverage-identity-growth-against-the-program-identity-bound, decide-whether-executable-coverage-evidence-folds-as-a-digest, attribute-the-canonical-manifest-growth-and-decide-whether-the-encoding-owes-a-budget, add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral]
scopes: [research/program-planning]
shared_scopes: [project/tickets]
paths: []
tags: [research, program-planning, identity, measurement]
claimed_from: todo
assignee: agent-ladder
lease_expires_at: 1786070865
---
## User-visible outcome

[`spikes/program-planning/identity-growth`](../spikes/program-planning/identity-growth/README.md) runs again and its ladder covers the domain the governed budget actually admits, so the record's headline claim — "the bound is unreachable for the program sizes this roadmap contemplates, with a margin of about 125×" — rests on measurement over the current domain rather than on an extrapolation from a domain that has since grown roughly eightfold.

## Why this exists — the harness says so itself, and it exits non-zero

**Measurement, 2026-08-06, at `f38813da`.** `cd spikes/program-planning/identity-growth && cargo run --release` **fails**, exit 1, on its own wall probe:

```
THE WALL MOVED: 9 operations compiled to a 44423-byte identity, so the governed
semantic-operations budget is no longer 8 and this ladder is no longer the whole
reachable domain. Widen OPERATIONS and rerun; the recorded result and its verdict
are stale.
```

That is the harness's designed refusal working exactly as its README says it should: "If the probe ever *succeeds*, the run fails and says the recorded result is stale — a moved budget widens the domain and invalidates the ladder, which is a finding rather than a pass."

**Fact — what moved it.** `36d05128` (*Integrate the budgets widening D-18 decided*, 2026-08-05) raised `DeterministicBudgets::governed`'s `semantic_operations` from **8 to 62**, sizing the five program-scoped bounds "to the complete decoder-layer program, which is the largest program shape this profile may be asked to admit". The spike's `OPERATIONS` ladder and its `BEYOND_THE_WALL` probe still name the old wall.

**Measurement — two things the re-run establishes before the refusal, and both are worth keeping.** The whole ladder is **+9 bytes at every point** against the retained result — 8,546 → 8,555, 12,866 → 12,875, 38,486 → 38,495 — and the fitted constant term is **719 rather than 710**. Those nine bytes are exactly the publishing-copy step `f8dfa8f6` landed, attributed independently at the envelope layer by [Where the artifact envelope's fixed content came from](../docs/research/artifacts/manifest-fixed-content-growth.md). And the wall probe's own number is a **free validation of the fit one step outside its domain**: `134·9² + 3650·9 + 719 = 44,423`, which is the measured identity length to the byte.

## What this ticket owes

- `OPERATIONS` and `BEYOND_THE_WALL` moved to the current governed budget, with the ladder's own doc comment restating the derivation rather than a new constant appearing without one.
- A re-run and a newly retained result beside the existing one, which is evidence at its own commit and is not overwritten.
- The verdict re-derived: the fitted curve, the refusal point, and the ×125 margin all move, and the record's *Verdict* section says which of its numbers reproduced and which moved.
- **A stated feasibility boundary, because the ladder may not reach 62.** The retained `compile_ms` column runs 1, 1, 2, 2, 3, 7, 14 over 2..=8 — roughly doubling per operation at the top of the range — so a contiguous 2..=62 ladder may not be affordable. If it is not, the honest form is a stated sub-range with the wall probe at the first point beyond it and the reason recorded, not a silently truncated sweep. How far the ladder actually reaches is itself a measurement this ticket owes.

## Why it matters beyond tidiness

The refusal point, the margin, and the deferral triggers on [`decide-whether-executable-coverage-evidence-folds-as-a-digest`](decide-whether-executable-coverage-evidence-folds-as-a-digest.md) were all derived against a domain of 2..=8 operations and a budget of 8. The budget is now 62 — that is, the governed profile already admits, by size, the decoder-layer program the deferral's third trigger treats as a future contingency. Nothing here says the 64 MiB bound is in danger; what it says is that the numbers guarding it were computed against a wall that has moved.

## Explicit non-goals

Not moving `semantic_operations`. Not deciding the digest question, which is [`decide-whether-executable-coverage-evidence-folds-as-a-digest`](decide-whether-executable-coverage-evidence-folds-as-a-digest.md)'s. Not editing that deferral's triggers, which is [`add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral`](add-the-embedding-ceiling-trigger-to-the-coverage-digest-deferral.md)'s.

## Closes when

The harness runs to completion at the current budget, a new result is retained beside the old one, and the research record states which figures reproduced, which moved, and how far the ladder reached with the reason it stopped there.
