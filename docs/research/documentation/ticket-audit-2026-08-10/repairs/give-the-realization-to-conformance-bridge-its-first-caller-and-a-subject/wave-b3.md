Ticket: give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject/2dc22e20cf4d_c99ac54950f2.md
Pre-edit content hash (from ledger): 2dc22e20cf4db1222318c4bbc9d869562c473abd17246c0d890b751c546e9a3e
Post-edit content hash: 9e1ae4fec8f90ae087000860ed904e9b8e7b87afe1e7f0ee67c69b1c8d4488ac

Changes applied:
  - Added `route-the-bf16-vertical-s-declared-conformance-through-the-checked-bridge` to frontmatter `related` (graph hygiene; child already depends on this parent).
  - Struck Outcome live claim "`tiler-reference` names no region type" and added **Correction — 2026-08-10** narrowing to: crate names `VerifiedIndexRegion`; does not name `VerifiedScheduledRegion` required by `RealizationWitness::of`.
  - Struck Outcome subsection **"Also out of scope and left stale"** as a live claim (present-tense "still says" about docs and `bf16_vertical.rs`); rephrased as close-time historical gap.
  - Added **Correction — 2026-08-10** that the route child (`status: done`) closed the production caller (`bf16_vertical::conformance_of` via `RealizationWitness` + `from_realization`) and repaired `docs/correctness-and-testing.md` and `bf16_vertical.rs`; restated this ticket's delivered boundary as in-crate bridge+subject+agreement+three-case restatement.

Optional items skipped (with reason):
  - none (related hygiene applied)

Residuals not applied (docs/crates/new tickets/authority):
  - none required; Exact files listed only this ticket. Draft public boundary under ADR 0075 remains labelled draft until Tom accepts (unchanged; not re-audited for a separate acceptance record).

Verification:
  - files read:
    - tickets/give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject.md (full, pre/post)
    - audit report 2dc22e20cf4d_c99ac54950f2.md (full)
    - tickets/route-the-bf16-vertical-s-declared-conformance-through-the-checked-bridge.md (status done; Outcome; Fact audit on region-type wording)
    - docs/correctness-and-testing.md (anchor `Each of those clauses is now false` / post-bridge prose)
    - crates/tiler-conformance/src/bf16_vertical.rs (struck header + `from_realization` at production `conformance_of`)
    - crates/tiler-reference/src/oracle.rs (`VerifiedIndexRegion` import / evaluator)
    - crates/tiler-reference/src/conformance.rs (`from_realization` present)
  - checks:
    - child ticket frontmatter `status: done`, `dependencies: [give-the-realization-to-conformance-bridge-its-first-caller-and-a-subject]`
    - `rg`/`read`: docs and bf16_vertical no longer assert pre-bridge gap as live fact
    - `shasum -a 256` on edited ticket → post-edit hash above

Recommended next ledger state:
  integrated
