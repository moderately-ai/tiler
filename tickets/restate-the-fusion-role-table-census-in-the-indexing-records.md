---
id: restate-the-fusion-role-table-census-in-the-indexing-records
title: Restate the fusion-role table census in the indexing fusion-role records
status: in-progress
priority: p3
dependencies: []
related: [correct-the-one-region-premise-in-the-concatenate-absence-check, admit-a-fusion-role-for-the-sub-tensor-selection-slice, admit-a-fusion-role-for-the-sequence-extension-concatenate]
scopes: [research/indexing, contracts/navigation]
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
- **Verified — sub-tensor record was live-stale at this edit base.** It then had `disposition: "pending"`, `implementation_status: "not-started"`, check 1's eleven-key/no-slice statement, check 2's three-key/slice-absent statement, and the eleven-key correction-to-precedent restatement. The completed slice-admission ticket and the `Sub-tensor selection` support-matrix row establish that the role landed as R5, without a request-boundary or lowering claim.
- **Verified — concatenate record was live-stale at this edit base only at its current inventory statements.** It then made checks 1–2 read as eleven keys and three coordinate keys and said the registry “maps nine operation keys onto them.” Its check 5 was already dated as a resolution-only correction and remains outside this ticket.
- **False — the initial concatenate maturity classification.** `tickets/lower-the-concatenate-occurrence-through-partitioned-writes.md` is `done` and its Outcome records seven capabilities plus `IndexRealizationLaw::PartitionedConcatenate`, emitted/verified/refined at every arity; [`accept-the-partitioned-concatenate-realization-law`](accept-the-partitioned-concatenate-realization-law.md) is also `done` and accepted the public law. Under `docs/document-metadata.md`'s retained high-water-mark rule, the concatenate record is `adopted` / `implemented`, not `partially-adopted` / `partial`.
- **False — the initial catalog non-goal.** `docs/document-metadata.md` says checked-in catalogs restate the frontmatter behind them. `docs/research/README.md` still calls both records `pending`, so its two exact rows must move with their frontmatter. This is a bounded `contracts/navigation` addition, not an authority to edit the matrix or delivery graph.
- **False — the initial nine→eleven provenance.** At the historical commits, the role-table census is 9 at `e97cad88^`, 10 at concatenate landing `e97cad88`, and 11 at contraction landing `b66dedbc`; `softmax_f32_op()` is already among the original nine. The slice record's intermediate-restatement sentence must name concatenate plus tensor contraction, not concatenate plus softmax.
- **Verified — reporter premise, with the required expansion.** `tickets/correct-the-one-region-premise-in-the-concatenate-absence-check.md` already distinguishes its historical claims from the live fifteen-key/four-key state and assigns the census remainder. The metadata contract adds the two matching catalog rows; it does not change the ticket's inventory purpose or reopen check 5.

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

Both documents map to `research/indexing`; their two catalog rows map to `contracts/navigation` and must mirror the corrected frontmatter. The tickets that landed the roles held `implementation/compiler` and `contracts/navigation` and correctly refused to edit research records out of scope. The docs-only one-region premise ticket could restate check 5 but left the census drift as remainder. One remainder owns both records so the table count and arm membership stay coherent across the pair that cite each other.

## What this must do

1. Re-read `FusionNumericalCapabilities::governed` and the `CoordinateRelation` arm at the edit base; treat the live `roles.insert` count and arm keys as authority, not any count written in this ticket.
2. Restate sub-tensor checks 1–2, the eleven-key body restatement, and frontmatter `disposition` / `implementation_status` so they match the landed M4 role without rewriting the four-candidate elimination history as if it ran against today's table.
3. Restate concatenate checks 1–2 and the body nine-key inventory the same way; leave check 5 and the resolution-only arity conclusion untouched.
4. Correct the slice record's nine→eleven provenance: concatenate raised the historical count from nine to ten, and tensor contraction raised it from ten to eleven; the softmax was already in the nine-key base.
5. Update both exact research-catalog rows to the dispositions the frontmatter carries, as the metadata contract requires.
6. Prefer short dated restatement blocks that keep intermediate counts legible as history.

## Explicit non-goals

- Any change to `crates/` registration, arm membership, or tests.
- Reopening check 5's premise or the seven-capabilities-per-arity lowering conclusion.
- Matrix rung movement, delivery-graph O-06/O-07 cell prose, or catalog files other than the two exact research-catalog rows required to mirror this ticket's corrected frontmatter.
- Index-access lowering or scheduled-region vocabulary work.

## Closes when

Both research records' live-looking role-table counts and contraction-arm membership match `grep -c 'roles.insert('` and the four-key arm at the edit base; sub-tensor frontmatter no longer reads pending/not-started behind a landed role; concatenate check 5 still states resolution-only; and a reader cannot mistake an intermediate census for current tree state.

## Graph maintenance

- `research/indexing` for both paths under `docs/research/indexing/**`.
- `contracts/navigation` for the two corresponding rows in `docs/research/README.md`; this scope was added after the metadata contract audit found the original catalog exclusion false.
- Prior reporters: [`correct-the-one-region-premise-in-the-concatenate-absence-check`](correct-the-one-region-premise-in-the-concatenate-absence-check.md), [`admit-a-fusion-role-for-the-sub-tensor-selection-slice`](admit-a-fusion-role-for-the-sub-tensor-selection-slice.md). Concatenate role landing cited as the first intermediate restatement source: [`admit-a-fusion-role-for-the-sequence-extension-concatenate`](admit-a-fusion-role-for-the-sequence-extension-concatenate.md).

## Outcome

**Fact — live inventory restated without erasing history.** At edit base `916d877d7b103567a4709346b6d6672f2cb54e60`, `rg -c -F 'roles.insert(' crates/tiler-compiler/src/fusion_legality.rs` returns `15`; `is_exact_governed_same_family_pointwise`'s `CoordinateRelation` guard names exactly `reindex`, `broadcast`, `concatenate`, and `slice`. Both records now label their nine-key base and eleven-key intermediate counts as historical and state the live census separately; no four-candidate elimination was rewritten as though it ran against the newer table.

**Correction — maturity metadata follows the governing landing evidence.** The slice record is `adopted` / `implemented`: its role and exact-key decision landed as R5, while the record still says that lowering and request-boundary admission were not delivered. The concatenate record is also `adopted` / `implemented`: its M4 role landed as R5, and its M5 partitioned-write lowering landed with seven capabilities, the accepted `PartitionedConcatenate` law, and verified/refined regions at every admitted arity. Check 5 in the concatenate record remains the resolution-only correction this ticket was forbidden to reopen.

**Correction — navigation and historical provenance.** The two `docs/research/README.md` rows now mirror the adopted frontmatter, as required by the documentation-metadata contract. The slice record's dated nine→eleven account now names the actual additions: concatenate (9→10) and tensor contraction (10→11); `softmax_f32_op()` was already registered at the nine-key research base.

**Checks.** `make citations` passed; `tkt lint --format json` returned `ok: true`; `git diff --check` passed; and `tkt guard tkt/restate-the-fusion-role-table-census-in-the-indexing-records --format json` reported no under-declaration. The residual census intentionally finds only explicitly historical nine/eleven-key statements, each anchored to its research base or dated restatement, plus live fifteen-key/four-key restatements.

**Full-gate carry.** The requested green full-gate baseline is `0b0e6952`. The delta contains only the two `docs/research/indexing/**` records, their two `docs/research/README.md` catalog rows, and this ticket; it touches none of the invalidating paths in `AGENTS.md`, so the full gate is carried. `make citations` and `tkt lint` were rerun for this documentation-only delta.
