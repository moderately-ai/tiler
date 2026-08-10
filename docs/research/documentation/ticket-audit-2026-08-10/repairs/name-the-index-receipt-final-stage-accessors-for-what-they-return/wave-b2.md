Ticket: name-the-index-receipt-final-stage-accessors-for-what-they-return
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/name-the-index-receipt-final-stage-accessors-for-what-they-return/a384717d1999_c99ac54950f2.md
Pre-edit content hash (from ledger): a384717d1999e0957376d37afe1a345f26836ecb955fb78e9f3d6b4d79b1d216
Post-edit content hash: 1200da578d3e66e92b92a33224587fc01cdec4e996087c2604195a02b0bcf606

Changes applied:
  - Outcome "every in-tree consumer": dropped fragile "Six call sites" / "×3" multiplicity; rephrased as named same-file tests plus residual-budget `proofs_for(unprovable.final_stage())` and one compiler forwarder.
  - Same paragraph: replaced `legality.rs:393` with searchable anchor `self.receipt.final_scalar_authority()` on `PendingIndexRefinement::scalar_authority` in `crates/tiler-compiler/src/legality.rs` (verified at current line 415).
  - Same paragraph: replaced `model.rs:1118` / `scalar.rs:2275` with anchors `pub const fn region` on `VerifiedScheduledRegion` / `ScalarAuthorityEvidence` (verified at current model.rs:1287 and scalar.rs:2510).
  - Outcome `const fn` constraint note: aligned the remaining reach description to the same post-rename anchor (`self.receipt.final_scalar_authority()`) so the retired `:393` / pre-rename call spelling does not linger.
  - Optional consistency: dated perturbation FAIL lines `refinement.rs:4486` / `:5524` as branch-local at Outcome time (2026-08-06), not stable locators.

Optional items skipped (with reason):
  - none (optional perturbation dating applied as cheap same-ticket hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by the report; no docs/crates edits, no remainder tickets, no metadata changes.
  - Related accept ticket still describing pre-rename `region()`/`scalar_authority()` surface language remains out of this ticket's edit scope (report residual uncertainty only).

Verification:
  - files read:
    - tickets/name-the-index-receipt-final-stage-accessors-for-what-they-return.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/name-the-index-receipt-final-stage-accessors-for-what-they-return/a384717d1999_c99ac54950f2.md
    - crates/tiler-compiler/src/legality.rs (forwarder at self.receipt.final_scalar_authority())
    - crates/tiler-ir/src/index/refinement.rs (named tests / final-stage call sites)
    - crates/tiler-ir/src/schedule/model.rs and crates/tiler-ir/src/index/scalar.rs (unrelated region accessors via rg)
  - checks:
    - rg confirmed `self.receipt.final_scalar_authority()` at legality.rs:415
    - rg confirmed `pub const fn region` at model.rs:1287 and scalar.rs:2510
    - rg over receipt final-stage consumers matches the rephrased inventory
    - frontmatter/status/deps/scopes/related left unchanged (report: none required)

Recommended next ledger state:
  integrated
