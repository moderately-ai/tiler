Ticket: refresh-the-l2-derivation-operation-family-standing
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/refresh-the-l2-derivation-operation-family-standing/e85db09a4e13_c99ac54950f2.md
Pre-edit content hash (from ledger): e85db09a4e1370ed92fcaab349255c9996cffccf803ecd61d5a81d05e1ca0df3
Post-edit content hash: 4f5d2976e7f26d4f0f9d404d986d26595ab1db5b64ffa120cfc950ed8b60ab05

Changes applied:
  - Kept `status: done` and added `realign-the-l2-derivation-operation-family-standing-to-the-family-state-table` to `related`.
  - Filed narrow remainder ticket `tickets/realign-the-l2-derivation-operation-family-standing-to-the-family-state-table.md` (`todo`, `research/shapes` + shared `project/tickets`) owning Softmax bound, Slice R5/fusion, Gather R4/key, and Concatenate/RMS bound re-read; linked back to parent.
  - Added dated **Correction — 2026-08-10** under Outcome remainder: quotes `No remaining correction is owned by this completed L2 refresh`, states Softmax/Slice/Gather re-stale against family-state table and registry, points at the remainder as owner; does not reopen parent or edit L2 document cells.

Optional items skipped (with reason):
  - none (no optional-only graph hygiene beyond the required related remainder link).

Residuals not applied (docs/crates/new tickets/authority):
  - L2 document standing cells in `docs/research/shapes/transformer-operation-and-shape-surface.md` (Softmax law/missing-capability bound; Slice R5 + CoordinateRelation; Gather R4 + registered key; "Nothing lowers, fuses, or emits either"; *The Rung column restated* gather R1; optional RMS bound re-read) — owned by the new remainder under `research/shapes`, not wave-B product edits on this ticket.
  - `crates/tiler-compiler/src/request.rs` module-doc sentence that softmax "carries no law at all" (audit residual outside this ticket's scopes).
  - Adjacent L1 handoff Slice R4 drift (outside `research/shapes`).

Verification:
  - files read: audit report; full parent ticket; remainder sibling `refresh-the-l2-derivation-s-symbolic-index-profile-source-claims.md` frontmatter/style; roadmap family-state Softmax/Slice/Gather cells; `registry.rs` standard law loop (15 laws including `staged_softmax_f32`); `softmax_recognizer_boundary.rs` (`missing-capability`); `fusion_legality.rs` `slice_f32_op` → `CoordinateRelation`; `gather.rs` `gather_f32_op` / `register_standard_gather`.
  - checks: `shasum -a 256` of parent ticket post-edit; re-verified audit Facts 3/5/6/12 authorities at current tree.

Recommended next ledger state:
  integrated
