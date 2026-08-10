Ticket: correct-the-documents-that-still-say-the-cache-root-preflight-is-never-called
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/correct-the-documents-that-still-say-the-cache-root-preflight-is-never-called/080820aacadf_c99ac54950f2.md
Pre-edit content hash (from ledger): 080820aacadfc8c08b022f2e30821727e8753510a721153849257477799be96e
Post-edit content hash: 8c26ce5b0cf98f637c0ca8ad0a718dd36572195cc30e831162b5834214cb9737

Changes applied:
  - Why this exists: dated Correction — 2026-08-10 on the two status/open-questions "still not called" Facts (filing motivation; live claims are 2026-08-05 successors) and on the frontends silence Fact (probe paragraph now present)
  - Non-goals: named ADR 0089 / root-policy as out of Required delivery and scopes
  - Closes when: narrowed to the three documents in User-visible outcome / Required delivery (option a — universal "no document" overclaim removed)
  - Outcome — landed by 2026-08-05 (ticket body recorded 2026-08-10): names status, open-questions, frontends and their anchors
  - Fact audit — 2026-08-10: code path verified; three-doc delivery verified; residual ADR/research live false claims; status done retained
  - metadata: status/related/scopes left unchanged (report: none required to keep done)

Optional items skipped (with reason):
  - option (b) connect a remainder ticket on related: no concrete remainder id in the report; would need a new ticket id decision (blocked residual below)

Residuals not applied (docs/crates/new tickets/authority):
  - docs/decisions/0089-resolve-the-expansion-cache-root-from-an-override-or-the-user-cache.md Consequences bullet: `ExpansionCache::preflight` is still not called on a resolved root — needs dated correction or rewrite (wave B ticket-only)
  - docs/research/cache/root-policy.md Unsupported cases: `ExpansionCache::preflight` is not called — same
  - blocked residual: new remainder ticket (or reopen with scopes `contracts/decisions` + research path) to land those two corrections, related to this ticket and `call-the-expansion-cache-preflight-on-the-resolved-root` — report requires filing but lists no concrete id; not created in this wave

Verification:
  - files read:
    - audit report 080820aacadf_c99ac54950f2.md (full)
    - tickets/correct-the-documents-that-still-say-the-cache-root-preflight-is-never-called.md (full, pre/post)
    - tickets/call-the-expansion-cache-preflight-on-the-resolved-root.md (frontmatter through Provenance)
    - greps: report_unsuitable_root in crates/tiler-macros/src; Corrected 2026-08-05 / probe anchors in docs/status.md, docs/open-questions.md, docs/integration/frontends.md; preflight still-not-called in ADR 0089 and root-policy.md
  - checks:
    - report_unsuitable_root: definition preflight.rs + call aot.rs open_cache Directory arm
    - status + open-questions: Corrected 2026-08-05 successors present
    - frontends.md: `The resolved root is probed in that same shape`
    - ADR 0089 + root-policy: live uncalled assertions still present (residual)
    - shasum -a 256 post-edit ticket

Recommended next ledger state:
  integrated
