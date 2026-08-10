---
id: refresh-the-reduced-precision-float-matrix-row-after-the-bf16-gate-landings
title: Refresh the reduced-precision float matrix row after the BF16 gate landings
status: done
priority: p3
dependencies: []
related: [own-operation-family-support-matrix, admit-a-bf16-scalar-arithmetic-subject, declare-the-bf16-rows-on-the-authoritative-metal-profile, derive-dtype-family-research-tracks-from-the-mature-taxonomy]
scopes: [contracts/navigation]
shared_scopes: [project/tickets]
paths: []
tags: [documentation, roadmap, bf16, matrix]
---
## User-visible outcome

The operation-family support matrix's reduced-precision float row names the gates that are actually open, so a worker reading it does not claim work that landed two days earlier.

## The exact drift

**Fact (at open, pre-`82b82edf`).** `docs/roadmap.md`'s `Arithmetic over reduced-precision floats` row closed with "BF16's remaining rungs are gated by `admit-a-bf16-scalar-arithmetic-subject` (R5/R6, the `ScalarArithmetic` constructor) and `conform-the-bf16-vertical-end-to-end` (R7, one run from semantic construction to a dispatched device result against this oracle)". That closing sentence is not live content after delivery. Reproduce the pre-landing sentence with `git show 82b82edf^:docs/roadmap.md | grep -n "remaining rungs are gated by"` (hits); `grep -n "remaining rungs are gated by" docs/roadmap.md` at the post-landing tree is empty. Do not use `grep -n "admit-a-bf16-scalar-arithmetic-subject" docs/roadmap.md` as proof of the old closing sentence — that id still appears once in the row as historical board derivation ("both gates this sentence used to name for R5/R6 … are `done`"), so the match exits 0 without proving the gated-by claim.

**Fact.** [`admit-a-bf16-scalar-arithmetic-subject`](admit-a-bf16-scalar-arithmetic-subject.md) is `done`, and [`declare-the-bf16-rows-on-the-authoritative-metal-profile`](declare-the-bf16-rows-on-the-authoritative-metal-profile.md) is `done`. [The dtype support ledger](../docs/dtype-support.md) records both, in two dated corrections: `ScalarArithmetic::new` is a public validated route proving the association from the governed built-in scalar catalog, and `BoundMetalCompileDeclaration` declares BF16 `Dispatchable` on `tiler.metal.macos-apple9.msl4-0.f32-bf16.v1` with complete exclusive subnormal tables.

**Inference (at open).** The row's gate list was stale relative to those landings, and whether the rung itself had moved was a separate question this ticket had to answer by reading rather than assume — the ledger promotes exactly two cells at one profile row and explicitly leaves the public `ScalarArithmetic::new` boundary a tested draft awaiting Tom under [ADR 0075](../docs/decisions/0075-scope-public-boundary-approval-by-change-category.md). A rung claim and a gate list are different assertions and this ticket must not conflate them. (`ScalarArithmetic::new` remains outside the `4ad5a2e` acceptance in `crates/tiler-compiler/src/target.rs` module docs.)

## Implementation keys

- Determine the current R5/R6 state by reading the construction sites, not by inferring from the closed tickets. The ledger names them; read `crates/tiler-compiler/src/target.rs` and the `tiler-build` declaration before writing a rung.
- Update the gate list to the tickets that are actually open, and state what each remaining gate owns.
- Do not move a [dtype support ledger](../docs/dtype-support.md) cell. That document owns the per-layer state and this row cites it.
- Check the row's other reduced-precision members are untouched: f16, the OFP8 pair, and the MX constituents have moved nothing.

## Closes when

The row's gate list matches the board, any rung claim is supported by a construction site named in the row, `tkt lint` and `git diff --check` pass.

## Outcome — 2026-08-10

**Delivery.** Landed at `82b82edf` ("Name the BF16 gates that are actually open so the matrix row stops sending workers at closed tickets"); closed on the board at `5f810e9a`. Edit set: `docs/roadmap.md` (one reduced-precision row cell rewrite), this ticket, and spin-off `correct-the-discharged-bf16-target-profile-claim-in-compiler-docs` (filed at delivery; later `closed` with `closed_reason: obsolete` because the docs were already corrected elsewhere).

**What the row received at close.** The false "remaining rungs are gated by …" closing sentence was replaced with a board-derived gate list of eight live tickets, each named with ownership, under the heading "BF16's remaining gates, derived from the board on 2026-08-05". R5 through R7 were rechecked at the construction sites and left unmoved at this ticket's close — closed gate tickets alone did not promote a rung. No `docs/dtype-support.md` cell was edited. Other reduced-precision members (f16, OFP8, MX constituents) were reread as untouched rather than carried forward.

**Post-close row maintenance (not residual of this ticket).** Later landings continued the same cell with dated recounts (2026-08-05 evening six, 2026-08-06 two, 2026-08-07 zero) and R5/R7 promotion language when those rungs actually moved. The live row's final recount ends "Recounted 2026-08-07: zero" with an explicit empty-list-does-not-mean-BF16-is-finished clause for unregistered reduction/contraction/conversion families. Those updates belong to the tickets that landed those rungs, not to a reopened remainder of this navigation refresh.

**Correction — 2026-08-10.** The open-time Facts above were written in present tense and stayed that way after close without an Outcome section. This section is the terminal record: `status: done` remains correct for the authorized work (refresh the gate list; do not promote ledger cells; do not invent rung movement). The pre-landing closing sentence must not be read as current roadmap content.
