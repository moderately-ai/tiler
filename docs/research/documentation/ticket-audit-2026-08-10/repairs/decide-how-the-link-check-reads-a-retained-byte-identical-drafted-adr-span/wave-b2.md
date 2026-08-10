Ticket: decide-how-the-link-check-reads-a-retained-byte-identical-drafted-adr-span
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/decide-how-the-link-check-reads-a-retained-byte-identical-drafted-adr-span/350b0a2f2c2d_c99ac54950f2.md
Pre-edit content hash (from ledger): 350b0a2f2c2d8020c240894422ba05120b7e2c71d7e031ce7a1c09bbd60b5434
Post-edit content hash: 62196af43b8aedae123677677c1c78753e43d2942a12b5ff88120103ab99d845

Changes applied:
  - Body quote pin: retired continuous strings under the drafted-span refusal paragraph pinned to filing base `db3f4d07` instead of present-tense "in those words".
  - Dated correction `**Correction — 2026-08-10.**` with live runtime and numerics refusal sentences after the fence rewrite (`91f67cc5` / `6ddcb305`).
  - Outcome narrowed: conversion-pair ticket owns opening-fence-removal and post-fence broken-link perturbations; runtime ticket credited only for fence + AGENTS.md repair + green check at `91f67cc5`/`c8c3da05`.
  - Outcome provenance tidy (optional, applied): name content parent `91f67cc5` beside merge `b118a4af` for the runtime fence.
  - Outcome paragraphs unwrapped (no mid-paragraph hard wrap).

Optional items skipped (with reason):
  - none (report optional provenance tidy applied on the same Outcome edit).

Residuals not applied (docs/crates/new tickets/authority):
  - none for the decision subject; report listed no docs/crates edits and no new remainder.
  - Body still notes the historical AGENTS.md overstatement as "a separate repair under `research/runtime`" (report fact 6 marks that repair already delivered; not in required Repair bullets; left as historical narrative).

Verification:
  - files read:
    - tickets/decide-how-the-link-check-reads-a-retained-byte-identical-drafted-adr-span.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/decide-how-the-link-check-reads-a-retained-byte-identical-drafted-adr-span/350b0a2f2c2d_c99ac54950f2.md
    - docs/research/runtime/backend-scoped-route-requirement-answers.md (live "Repointing them here is still refused…" sentence)
    - docs/research/numerics/conversion-family-decomposition-across-pairs.md (live "Repointing them here is refused…" sentence)
    - tickets/repair-the-eight-dangling-links-in-the-runtime-route-answer-record.md (Outcome: fence at `91f67cc5`, close at `c8c3da05`; no fence-perturbation text)
    - tickets/repair-the-two-dangling-adr-links-in-the-conversion-pair-record.md (`Opening fence removed` perturbation evidence)
  - checks:
    - live repoint refusal strings present in both research records; retired continuous quotes absent from current tree
    - conversion-pair ticket records subject perturbations; runtime ticket does not
    - `shasum -a 256` on ticket after edit

Recommended next ledger state:
  integrated
