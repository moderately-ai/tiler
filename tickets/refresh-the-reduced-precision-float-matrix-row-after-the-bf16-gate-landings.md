---
id: refresh-the-reduced-precision-float-matrix-row-after-the-bf16-gate-landings
title: Refresh the reduced-precision float matrix row after the BF16 gate landings
status: in-progress
priority: p3
dependencies: []
related: [own-operation-family-support-matrix, admit-a-bf16-scalar-arithmetic-subject, declare-the-bf16-rows-on-the-authoritative-metal-profile, derive-dtype-family-research-tracks-from-the-mature-taxonomy]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, roadmap, bf16, matrix]
claimed_from: todo
assignee: agent-bf16-matrix
lease_expires_at: 1785941396
---
## User-visible outcome

The operation-family support matrix's reduced-precision float row names the gates that are actually open, so a worker reading it does not claim work that landed two days earlier.

## The exact drift

**Fact.** `docs/roadmap.md`'s `Arithmetic over reduced-precision floats` row closes with "BF16's remaining rungs are gated by `admit-a-bf16-scalar-arithmetic-subject` (R5/R6, the `ScalarArithmetic` constructor) and `conform-the-bf16-vertical-end-to-end` (R7, one run from semantic construction to a dispatched device result against this oracle)". Reproduce with `grep -n "admit-a-bf16-scalar-arithmetic-subject" docs/roadmap.md`.

**Fact.** [`admit-a-bf16-scalar-arithmetic-subject`](admit-a-bf16-scalar-arithmetic-subject.md) is `done`, and [`declare-the-bf16-rows-on-the-authoritative-metal-profile`](declare-the-bf16-rows-on-the-authoritative-metal-profile.md) is `done`. [The dtype support ledger](../docs/dtype-support.md) records both, in two dated corrections: `ScalarArithmetic::new` is now a public validated route proving the association from the governed built-in scalar catalog, and `BoundMetalCompileDeclaration` declares BF16 `Dispatchable` on `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` with complete exclusive subnormal tables.

**Inference.** The row's gate list is stale, and whether the rung itself moved is a separate question this ticket must answer by reading rather than assume — the ledger promotes exactly two cells at one profile row and explicitly leaves the public `ScalarArithmetic::new` boundary a tested draft awaiting Tom under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md). A rung claim and a gate list are different assertions and this ticket must not conflate them.

## Implementation keys

- Determine the current R5/R6 state by reading the construction sites, not by inferring from the closed tickets. The ledger names them; read `crates/tiler-compiler/src/target.rs` and the `tiler-build` declaration before writing a rung.
- Update the gate list to the tickets that are actually open, and state what each remaining gate owns.
- Do not move a [dtype support ledger](../docs/dtype-support.md) cell. That document owns the per-layer state and this row cites it.
- Check the row's other reduced-precision members are untouched: f16, the OFP8 pair, and the MX constituents have moved nothing.

## Closes when

The row's gate list matches the board, any rung claim is supported by a construction site named in the row, `tkt lint` and `git diff --check` pass.
