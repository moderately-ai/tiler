Ticket: re-transfer-the-adr-0092-span-after-the-item-6-restatement-fork
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/re-transfer-the-adr-0092-span-after-the-item-6-restatement-fork/75f4f8b965c2_c99ac54950f2.md
Pre-edit content hash (from ledger): 75f4f8b965c23ead2205cb90eaac418c00f3437f9e17bfe0d5d2e04ff88417c9
Post-edit content hash: cf740cd03bf4f9e8f28ffbe208296a508b1364c8bf0082665e2537513a4b9087

Changes applied:
  - dependencies: added `correct-adr-0092-item-6-s-widening-restatement-trap-and-its-retired-sentence-shorthand` (historical predecessor; was related-only); removed it from `related` to avoid double-list; left `decide-how-the-link-check-reads-a-retained-byte-identical-drafted-adr-span` in `related`
  - opening: rewrote present-tense "currently incomplete" into filing-time historical framing; points to Outcome for delivery
  - Facts: past-tense "forked position was" (no longer live present fork)
  - typo: `a a \`text\` code fence fence` → `a \`text\` code fence`
  - Outcome — 2026-08-08, `23746b12`: splice of item 6; `cmp` `differ: char 3051, line 19` → exit 0 over Context..Traceability (notes excluded, `###`→`##`); fence and in-span link spellings preserved; no in-place span edit
  - Correction — 2026-08-10: notes status `done` matches tree; architecture / public-boundary remain out of this close condition

Optional items skipped (with reason):
  - none (optional dependency graph hygiene applied)

Residuals not applied (docs/crates/new tickets/authority):
  - none required by this report for the re-transfer work; architecture item-6 amendment and public-boundary acceptance remain other tickets' obligations (named in Outcome, not filed here)
  - full `./check-citations.sh` population not re-run (audit residual; fence structure already verified at audit)

Verification:
  - files read:
    - tickets/re-transfer-the-adr-0092-span-after-the-item-6-restatement-fork.md (pre/post)
    - audit report 75f4f8b965c2_c99ac54950f2.md
    - tickets/correct-adr-0092-item-6-s-widening-restatement-trap-and-its-retired-sentence-shorthand.md (Outcome cites this ticket)
    - tickets/re-transfer-the-adr-0092-span-for-its-drifted-prototype-referent.md (Outcome house style)
  - checks:
    - `git merge-base --is-ancestor 23746b12 HEAD` → true
    - `git show 23746b12` message records failing-then-clean cmp and one-line splice; single-file stat on runtime record

Recommended next ledger state:
  integrated
