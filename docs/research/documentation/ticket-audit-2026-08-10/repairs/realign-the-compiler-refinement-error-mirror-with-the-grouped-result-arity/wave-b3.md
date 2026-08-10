Ticket: realign-the-compiler-refinement-error-mirror-with-the-grouped-result-arity
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/realign-the-compiler-refinement-error-mirror-with-the-grouped-result-arity/8c664f1f9d21_c99ac54950f2.md
Pre-edit content hash (from ledger): 8c664f1f9d21c162672ae8eca7bf4e0534c4599d1ddc2fe6ddf2433602a7477a
Post-edit content hash: 5a00d8df7f1bddb909b6ead0151dd85fc7611a0e5871a32b9fac5d2ad8b73cb2

Changes applied:
  - related: added `realign-the-result-side-display-strings-across-both-refinement-error-mirrors` so the Outcome Display handoff is discoverable from this ticket (optional graph hygiene; reverse related already existed on the child).
  - Why this exists / "no observable value changed" Fact: struck live present-tense "and no registered capability emits a partitioned region yet"; **Correction — 2026-08-10** notes `GovernedConcatenateF32` / partitioned concatenate landed with `PartitionMember` ownership proofs; kept the test-assertion Fact that `a_well_formed_region_with_an_extra_output_is_rejected` still observes `region_outputs: 2`.
  - Outcome observation / Display handoff: **Correction — 2026-08-10** records that the Display remainder was filed as `realign-the-result-side-display-strings-across-both-refinement-error-mirrors` (`status: todo`), so "not filed, left to the coordinator" is close-time history not current graph state; also notes stale line-number citations vs symbol/quoted-string anchors.

Optional items skipped (with reason):
  - none (optional related add and optional stale-line note both applied as cheap same-ticket hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - none for this wave — Display work already ticketed; compiler source already matches closed Outcome; no new remainder required.

Verification:
  - files read: assigned audit report; full ticket pre-edit; Display remainder ticket frontmatter; `governed.rs` GovernedConcatenateF32 docs + PartitionMember site; legality.rs ResultArity field doc + Display incomplete-write string; IR refinement Display twin string.
  - checks: `GovernedConcatenateF32` present with partitioned-write docs; Display child ticket `status: todo` with reverse related; pre-edit hash matches ledger; post-edit sha256 computed.

Recommended next ledger state:
  integrated
