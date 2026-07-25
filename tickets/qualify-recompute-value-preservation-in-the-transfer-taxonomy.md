---
id: qualify-recompute-value-preservation-in-the-transfer-taxonomy
title: Qualify recompute value preservation in the transfer taxonomy
status: in-progress
priority: p3
dependencies: []
related: [reconcile-the-transfer-taxonomy-convertdtype-label-with-the-enforcer-definition, transfer-synchronization-and-resource-lifetime-contract]
scopes: [research/transfers]
shared_scopes: [project/tickets]
paths: []
tags: [research, transfers, numerics]
claimed_from: todo
assignee: agent-qualify-recompute
lease_expires_at: 1785039083
---
`reconcile-the-transfer-taxonomy-convertdtype-label-with-the-enforcer-definition` checked every `PlacementEnforcer` variant in `docs/research/transfers/transfer-synchronization-and-resource-lifetime.md` against the accepted definition in `docs/compiler/optimizer.md` — that an enforcer "may change only how a boundary value is stored, addressed, placed, or delivered, never which values that boundary carries". Eight of the nine settled immediately. `Recompute(RecomputeStage)` did not, and it is the one variant the memo's "Enforcer and mechanism taxonomy" table never describes at all.

Every other enforcer moves, aliases, migrates, re-lays-out, or repacks an authoritative value version that already exists; its value preservation is discharged by the mechanism. Recomputation does not move a version, it re-derives one. Whether the re-derived value is the same value is a numerical question, not a placement one: a recomputation whose reduction order, contraction, or subnormal treatment differs from the original produces a different value, and under the very same ADR 0001 argument the definition rests on, one semantic program would then mean different things under two plans that differ only in whether a value was recomputed or kept.

ADR 0047 accepted recomputation as an enforcer and that acceptance is preserved — this is not a proposal to remove the variant. What is missing is the obligation that makes it legitimate. Decide and record:

- what `RecomputeStage` must carry so a verifier can prove the re-derived value equals the retained one under the effective numerical contract, and whether that is bit-identity or a stated weaker relation the boundary's requester declared acceptable;
- whether the obligation is discharged structurally — the recomputation is the same `ScheduledRegion` identity with the same numerical realization, which would make it checkable without a value model — or whether it needs a numerical proof of the kind `FusionNumericalProof` already carries;
- whether a recomputation that cannot discharge it is a typed feasibility rejection rather than a cost, per the repository's rule that hard feasibility is never hidden behind an infinite cost; and
- a row in the taxonomy table for `Recompute`, since its absence is why the gap survived a table that exists to make exactly this comparison.

The memo is a proposal that no accepted ADR and no normative contract has incorporated, so this is a gap to close before incorporation rather than a defect in an accepted contract.

Closes when the taxonomy table describes `Recompute`, its value-preservation obligation is stated, and `uv run --locked python scripts/check_repository.py` passes.
