Ticket: call-the-expansion-cache-preflight-on-the-resolved-root
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/call-the-expansion-cache-preflight-on-the-resolved-root/55fd1387a3e6_c99ac54950f2.md
Pre-edit content hash (from ledger): 55fd1387a3e66f981f2bcd1b6125ba9c6c3b1a2be5a2cd9b266148f319b193c5
Post-edit content hash: 41aaadb06bc43de6c229a3e963580f4f078187e0a90cbe9340416d89eb3cd9ee

Changes applied:
  - Why this exists: **Correction — 2026-08-05 (delivered; ticket body recorded 2026-08-10)** on the obsolete "never taken / only doc comment" Fact; line anchors marked historical; Inference updated to name residual ADR/root-policy debt and the remainder ticket
  - Diagnostic-vs-correctness framing retained as still-true after delivery (probe never maps to AotRefusal)
  - Required delivery: line-number doc cite on cache_root replaced with symbol `checked`
  - Non-goals: ADR 0089 / root-policy named out of scope
  - Outcome — delivered 2026-08-05: preflight module, open_cache call, gate, checked doc, watched tests, pointer to three-doc sibling; reproduce via report_unsuitable_root
  - Fact audit — 2026-08-10: delivery verified; residual owned by remainder
  - related: added correct-adr-0089-and-root-policy-preflight-uncalled-claims
  - status done retained (no metadata status change required)

Optional items skipped (with reason):
  - none material; dependencies left empty (API already existed; report accepted [])

Residuals not applied (docs/crates/new tickets/authority):
  - docs product debt is now ticketed, not edited in this wave: docs/decisions/0089-... Consequences + docs/research/cache/root-policy.md Unsupported cases (scopes contracts/decisions + research/cache)
  - new remainder filed: tickets/correct-adr-0089-and-root-policy-preflight-uncalled-claims.md (todo; related to this ticket and correct-the-documents-that-still-say-the-cache-root-preflight-is-never-called)
  - sibling docs ticket related list not edited (hard rule: do not touch other existing tickets); remainder already lists it under related for reverse discovery

Verification:
  - files read:
    - audit report 55fd1387a3e6_c99ac54950f2.md (full)
    - tickets/call-the-expansion-cache-preflight-on-the-resolved-root.md (full, pre/post)
    - tickets/correct-the-documents-that-still-say-the-cache-root-preflight-is-never-called.md (full)
    - tickets/correct-the-uncalled-cache-root-resolver-claim.md (pattern for remainder)
    - cache_root.rs checked doc; greps for report_unsuitable_root, ADR/root-policy preflight sentences
    - ticketsplease.toml scopes for contracts/decisions and research/cache
  - checks:
    - report_unsuitable_root: def preflight.rs + call aot.rs open_cache Directory arm
    - cache_root::checked doc points at crate::preflight
    - ADR 0089 line with still-not-called + root-policy is-not-called still present (remainder work)
    - shasum -a 256 post-edit parent ticket

Recommended next ledger state:
  integrated
