Ticket: scope-the-padding-and-cropping-family
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-padding-and-cropping-family/92f7c1b352a8_c99ac54950f2.md
Pre-edit content hash (from ledger): 92f7c1b352a8ad2983e0591d30502b901264eb8ff8f185aa93c37b2c83ac2e6f
Post-edit content hash: ff75c5d89fff54c4252002a895480b4bc40cb7c74ff2df833b458bf15feea226

Changes applied:
  - Rewrote the second Fact paragraph under "Why this is deferred rather than open": replaced the false citation of the support-matrix contraction row with roadmap Milestone 6 **Fact — K-padding is not free, and the contract already says so.** (numerical-semantics remains the primary three-way-split authority already cited in the same paragraph). Substance retained: the ragged-K tile-pad neutrality obligation is the same rule.
  - Appended optional 2026-08-10 trigger-check log line (**not fired**); notes key-census drift so 2026-08-05 "46/18" is not read as live.

Optional items skipped (with reason):
  - none (optional trigger log and clean rewrite without dated correction note both applied; geometric-resampling back-link not required per report).

Residuals not applied (docs/crates/new tickets/authority):
  - none (report required ticket prose/citation only; no docs/crates remainder, no new tickets).

Verification:
  - files read:
    - tickets/scope-the-padding-and-cropping-family.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-padding-and-cropping-family/92f7c1b352a8_c99ac54950f2.md
    - docs/roadmap.md (Milestone 6 K-padding framing)
    - docs/numerical-semantics.md (Empty domains three-way split)
  - checks:
    - roadmap anchor present: `**Fact — K-padding is not free, and the contract already says so.**` and `Padding the contracted extent to a tile multiple with zeros`
    - numerical-semantics: `Empty result, algebraic identity, and safe physical padding are separate facts.`
    - no pad/crop OpKey under semantic construction sites (grep); status deferred left unchanged

Recommended next ledger state:
  integrated
