---
id: restate-the-fusion-role-table-census-in-the-indexing-records
title: Restate the fusion-role table census in the indexing fusion-role records
status: in-progress
priority: p3
dependencies: []
related: [correct-the-one-region-premise-in-the-concatenate-absence-check, admit-a-fusion-role-for-the-sub-tensor-selection-slice, admit-a-fusion-role-for-the-sequence-extension-concatenate]
scopes: [research/indexing]
shared_scopes: [project/tickets]
paths: []
tags: [documentation]
claimed_from: todo
assignee: terra-indexing-fusion-census
lease_expires_at: 1786412599
---
## What is stale

Two research/indexing fusion-role records still assert intermediate role-table censuses that the live `FusionNumericalCapabilities::governed` registration has outgrown. The drift is pure document inventory; it does not reopen any landed elimination conclusion about which role either family takes.

**Fact — live table (verify before editing; do not inherit a count from this ticket).** At the 2026-08-10 audit base `c99ac54950f2` and re-checked on the tree that filed this ticket: `grep -c 'roles.insert(' crates/tiler-compiler/src/fusion_legality.rs` returns **15**. The `CoordinateRelation` arm of `is_exact_governed_same_family_pointwise` matches four exact keys — `reindex_f32_op()`, `broadcast_f32_op()`, `concatenate_f32_op()`, and `slice_f32_op()`. Reproduce arm membership with `grep -n 'fn is_exact_governed_same_family_pointwise' -A 70 crates/tiler-compiler/src/fusion_legality.rs` and confirm the four-way match, not a line pin.

### Sub-tensor selection fusion role

[`docs/research/indexing/sub-tensor-selection-fusion-role.md`](../docs/research/indexing/sub-tensor-selection-fusion-role.md):

- Frontmatter still has `disposition: "pending"` and `implementation_status: "not-started"` after [`admit-a-fusion-role-for-the-sub-tensor-selection-slice`](admit-a-fusion-role-for-the-sub-tensor-selection-slice.md) landed the role and moved the matrix R5 cell.
- Reproducible check 1 still claims "eleven keys and no slice"; check 2 still claims the contraction arm is closed over three keys with the slice outside.
- The body restatement at the "correction to the precedent record" section (searchable anchor: `the live \`grep -c\` above returns \`11\``) still reports eleven as the live count.

Reported rather than absorbed by [`correct-the-one-region-premise-in-the-concatenate-absence-check`](correct-the-one-region-premise-in-the-concatenate-absence-check.md) (Out of scope at landing; **Correction — 2026-08-10** on that ticket freezes the present-tense remainder) and again by the slice admission ticket's Outcome flag.

### Concatenate fusion role and lowering

[`docs/research/indexing/concatenate-fusion-role-and-lowering.md`](../docs/research/indexing/concatenate-fusion-role-and-lowering.md):

- Reproducible check 1 still claims "eleven keys and the concatenate is among them"; check 2 still claims the arm is closed over three keys with concatenate among them.
- Body Fact still says `FusionNumericalCapabilities::governed()` "maps nine operation keys onto them" (searchable anchor: `maps nine operation keys onto them`).
- **Do not reopen check 5.** Check 5's premise was restated under [`correct-the-one-region-premise-in-the-concatenate-absence-check`](correct-the-one-region-premise-in-the-concatenate-absence-check.md) to signature-exact resolution alone; that subject is closed and must stay closed. Census restatement is checks 1–2 and the body key-count inventory only.

## Why it is a separate ticket

Both documents map to `research/indexing`. The tickets that landed the roles held `implementation/compiler` and `contracts/navigation` and correctly refused to edit research records out of scope. The docs-only one-region premise ticket could restate check 5 but left the census drift as remainder. One remainder owns both records so the table count and arm membership stay coherent across the pair that cite each other.

## What this must do

1. Re-read `FusionNumericalCapabilities::governed` and the `CoordinateRelation` arm at the edit base; treat the live `roles.insert` count and arm keys as authority, not any count written in this ticket.
2. Restate sub-tensor checks 1–2, the eleven-key body restatement, and frontmatter `disposition` / `implementation_status` so they match the landed M4 role without rewriting the four-candidate elimination history as if it ran against today's table.
3. Restate concatenate checks 1–2 and the body nine-key inventory the same way; leave check 5 and the resolution-only arity conclusion untouched.
4. Prefer short dated restatement blocks that keep intermediate counts legible as history (the pattern already used in those check blocks for the nine→eleven step).

## Explicit non-goals

- Any change to `crates/` registration, arm membership, or tests.
- Reopening check 5's premise or the seven-capabilities-per-arity lowering conclusion.
- Matrix rung movement, delivery-graph O-06/O-07 cell prose, or other catalog files outside these two research records (file separately if a cell still lags after this restatement).
- Index-access lowering or scheduled-region vocabulary work.

## Closes when

Both research records' live-looking role-table counts and contraction-arm membership match `grep -c 'roles.insert('` and the four-key arm at the edit base; sub-tensor frontmatter no longer reads pending/not-started behind a landed role; concatenate check 5 still states resolution-only; and a reader cannot mistake an intermediate census for current tree state.

## Graph maintenance

- `research/indexing` for both paths under `docs/research/indexing/**`.
- Prior reporters: [`correct-the-one-region-premise-in-the-concatenate-absence-check`](correct-the-one-region-premise-in-the-concatenate-absence-check.md), [`admit-a-fusion-role-for-the-sub-tensor-selection-slice`](admit-a-fusion-role-for-the-sub-tensor-selection-slice.md). Concatenate role landing cited as the first intermediate restatement source: [`admit-a-fusion-role-for-the-sequence-extension-concatenate`](admit-a-fusion-role-for-the-sequence-extension-concatenate.md).
