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

**Fact — live table (verify before editing; do not inherit a count from this ticket).** At the 2026-08-10 audit base `c99ac54950f2` and re-checked at this ticket's edit base `916d877d7b103567a4709346b6d6672f2cb54e60`: `rg -c -F 'roles.insert(' crates/tiler-compiler/src/fusion_legality.rs` returns **15**. The `CoordinateRelation` arm of `is_exact_governed_same_family_pointwise` matches four exact keys — `reindex_f32_op()`, `broadcast_f32_op()`, `concatenate_f32_op()`, and `slice_f32_op()`. Reproduce arm membership with `rg -n -F 'fn is_exact_governed_same_family_pointwise' -A 70 crates/tiler-compiler/src/fusion_legality.rs` and confirm the four-way match, not a line pin.

## Base Fact audit — 2026-08-10 at `916d877d7b103567a4709346b6d6672f2cb54e60`

- **Verified — live table.** `crates/tiler-compiler/src/fusion_legality.rs` contains 15 `roles.insert(` registrations, and `is_exact_governed_same_family_pointwise`'s `FusionOperationRole::CoordinateRelation` guard names exactly `reindex`, `broadcast`, `concatenate`, and `slice`. Earlier wording calling this the “contraction arm” was imprecise: it is the coordinate-relation arm of the arithmetic-contraction proof.
- **Verified — sub-tensor record is live-stale.** `docs/research/indexing/sub-tensor-selection-fusion-role.md` still has `disposition: "pending"`, `implementation_status: "not-started"`, check 1's eleven-key/no-slice statement, check 2's three-key/slice-absent statement, and the eleven-key correction-to-precedent restatement. The completed slice-admission ticket and the `Sub-tensor selection` support-matrix row establish that the role landed as R5, without a request-boundary or lowering claim.
- **Verified — concatenate record is live-stale only at its current inventory statements.** `docs/research/indexing/concatenate-fusion-role-and-lowering.md` still makes checks 1–2 read as eleven keys and three coordinate keys and says the registry “maps nine operation keys onto them.” Its check 5 is already dated as a resolution-only correction and remains outside this ticket.
- **Verified — reporter premise.** `tickets/correct-the-one-region-premise-in-the-concatenate-absence-check.md` already distinguishes its historical claims from the live fifteen-key/four-key state and assigns exactly this bounded remainder. No Fact changes this ticket's purpose, identity, or authority.

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

## Outcome

**Fact — live inventory restated without erasing history.** At edit base `916d877d7b103567a4709346b6d6672f2cb54e60`, `rg -c -F 'roles.insert(' crates/tiler-compiler/src/fusion_legality.rs` returns `15`; `is_exact_governed_same_family_pointwise`'s `CoordinateRelation` guard names exactly `reindex`, `broadcast`, `concatenate`, and `slice`. Both records now label their nine-key base and eleven-key intermediate counts as historical and state the live census separately; no four-candidate elimination was rewritten as though it ran against the newer table.

**Fact — maturity metadata follows the governing landing evidence.** The slice record is `adopted` / `implemented`: its role and exact-key decision landed as R5, while the record still says that lowering and request-boundary admission were not delivered. The concatenate record is `partially-adopted` / `partial`: its M4 role landed as R5 and its M5 lowering conclusion remains separately incomplete. Check 5 in the concatenate record remains the resolution-only correction this ticket was forbidden to reopen.

**Checks.** `make citations` passed; `tkt lint --format json` returned `ok: true`; `git diff --check` passed; and `tkt guard tkt/restate-the-fusion-role-table-census-in-the-indexing-records --format json` reported no under-declaration. The residual census intentionally finds only explicitly historical nine/eleven-key statements, each anchored to its research base or dated restatement, plus live fifteen-key/four-key restatements.

**Full-gate carry.** The requested green full-gate baseline is `0b0e6952`. `git diff --name-only 916d877d7b103567a4709346b6d6672f2cb54e60` contains only the two `docs/research/indexing/**` records and this ticket; it touches none of the invalidating paths in `AGENTS.md`, so the full gate is carried. `make citations` and `tkt lint` were rerun for this ticket-only/document-only delta.
