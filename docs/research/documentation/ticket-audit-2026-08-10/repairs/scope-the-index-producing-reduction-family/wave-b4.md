Ticket: scope-the-index-producing-reduction-family
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-index-producing-reduction-family/2827b9eabdbe_c99ac54950f2.md
Pre-edit content hash (from ledger): 2827b9eabdbe51d0637f8c10a78ce0220446f459cd1944ee7c03c3f9c52c4c4e
Post-edit content hash: 4a620cb4cc21979e1545929a7ae29c30957359bfe9de91de13b37a3591e11061

Changes applied:
  - Replaced attributed L6 quote "logits remain a consumer-sampled output" with L6's actual logits-contract wording (head output is logits; no argmax/token; sampling/greedy outside the graph; device-side argmax needs a non-existent max-with-index family).
  - Rewrote adjacent-permission Fact: admit-a-parallel-topology-for-the-identity-less-extrema-fold is done and delivered EmptyDomainContract::NoIdentity (non-empty domain proof, not carried has_value staged-partial); index-producing fold would consume that empty-domain / identity-less combine permission rather than F-28 monoid topology permissions.
  - Aligned "What the work would be" parallel-combine sentence to name EmptyDomainContract::NoIdentity instead of "staged-partial contract".
  - Added 2026-08-10 trigger-check log entry **not fired** with refreshed census (47 unique governed keys; 19 operation `*_op` functions; family's key absent). Left 2026-08-05/09 historical rows unchanged.

Optional items skipped (with reason):
  - Optional related-list annotation that the extrema-parallel ticket is done: prose already states done + delivered contract; frontmatter related ids unchanged and remain correct.
  - Optional mutual edge with monoid-reducers ticket: graph hygiene outside this ticket's required repair; monoid ticket already lists this one.

Residuals not applied (docs/crates/new tickets/authority):
  - none — Exact files listed only this ticket; no remainder filing required; product activation remains correctly deferred.

Verification:
  - files read:
    - tickets/scope-the-index-producing-reduction-family.md (full, before and after edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-index-producing-reduction-family/2827b9eabdbe_c99ac54950f2.md (full)
    - docs/research/program-planning/complete-model-ingestion-and-execution.md (L6 logits contract anchors)
    - tickets/admit-a-parallel-topology-for-the-identity-less-extrema-fold.md (status: done; EmptyDomainContract delivery)
    - crates/tiler-ir/src/schedule/builder.rs (EmptyDomainContract / NoIdentity)
    - crates/tiler-ir/src/semantic/* (op-key census)
  - checks:
    - `rg -n 'no argmax' docs/research/program-planning/complete-model-ingestion-and-execution.md` — L6 actual wording present; `consumer-sampled` absent
    - `rg -n 'pub fn \w+_op\(\) -> OpKey' crates/tiler-ir/src/semantic --glob '*.rs'` — 19 matches
    - `rg -o -N --no-filename 'tiler::[a-z0-9-]+@[0-9]+' crates/tiler-ir/src/semantic/ | sort -u` — 47 unique keys; no argmin/argmax index-producing key
    - admit ticket frontmatter `status: done`; EmptyDomainContract::NoIdentity in builder.rs
    - shasum -a 256 tickets/scope-the-index-producing-reduction-family.md → post-edit hash above

Recommended next ledger state:
  integrated
