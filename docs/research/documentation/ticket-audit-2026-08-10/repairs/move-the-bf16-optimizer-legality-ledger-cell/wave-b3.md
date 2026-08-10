Ticket: move-the-bf16-optimizer-legality-ledger-cell
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/move-the-bf16-optimizer-legality-ledger-cell/6ce12108d4ef_c99ac54950f2.md
Pre-edit content hash (from ledger): 6ce12108d4ef5cca9a77d14252c1d400d524c4c5e98fe61e95150411d6c4d652
Post-edit content hash: a588a5f06750e2e72816c13aacf192c27a224a486905f0da746c41f37994f168

Changes applied:
  - Rephrased § "Four items…" from present-tense "is unclaimed" / "Left for it" to past-tense closed handoff: sibling could not batch at delivery, is now `status: done`, and discharged the four items (Physical carrier, dtype-f32/recognizer-era prose, roadmap recheck).
  - Historical `dependencies: [establish-bf16-optimizer-legality]` (was empty; establish released this ticket; was only under `related`).
  - Dated correction under Outcome: "stays `Unknown`" named the wrong *obligation* outcome; permission remains withheld (`BF16_FACT_REASSOCIATION_PERMITTED` false); vacuous pointwise reassociation discharges `SoundProof`, not `Unknown`.

Optional items skipped (with reason):
  - none (optional dependency + reassociation dated note applied as cheap same-ticket hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - none; report required no docs/crates edits and no new remainder tickets

Verification:
  - files read:
    - audit report `6ce12108d4ef_c99ac54950f2.md` (full)
    - `tickets/move-the-bf16-optimizer-legality-ledger-cell.md` (full, pre/post)
    - sibling frontmatter `status: done` on `correct-the-stale-dtype-f32-recognizer-claims-in-the-contract-documents.md`
  - checks:
    - sibling still `status: done` at current tree
    - post-edit sha256 recomputed via `shasum -a 256`

Recommended next ledger state:
  integrated
