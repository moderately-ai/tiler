---
id: move-the-bf16-optimizer-legality-ledger-cell
title: Move the BF16 optimizer legality ledger cell
status: in-progress
priority: p3
dependencies: []
related: [establish-bf16-optimizer-legality]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [docs, support-matrix]
claimed_from: todo
assignee: agent-ledger
lease_expires_at: 1786132480
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
