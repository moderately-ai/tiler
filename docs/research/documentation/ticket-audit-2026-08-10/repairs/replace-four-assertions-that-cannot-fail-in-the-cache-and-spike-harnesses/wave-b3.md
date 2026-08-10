Ticket: replace-four-assertions-that-cannot-fail-in-the-cache-and-spike-harnesses
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/replace-four-assertions-that-cannot-fail-in-the-cache-and-spike-harnesses/97ef9d1fdf9c_c99ac54950f2.md
Pre-edit content hash (from ledger): 97ef9d1fdf9c1db091968412e2075abaf63c6a04ca15eebb49b3500f13a71419
Post-edit content hash: 5692b4a3102a62caabda2fc7c65bf3f41f2990617cb74b7c38cfa47dd9ab0bb6

Changes applied:
  - frontmatter `related: []` → `related: [pin-the-bf16-spike-admitted-operation-verdict-to-its-own-enum, correct-the-accounts-for-every-entry-claim-in-the-cache-research-note-and-harness]` (bidirectional discoverability for the two scope-split remainder tickets named in Outcome; both files exist on tree)

Optional items skipped (with reason):
  - Outcome count wording clarification ("11 tests call collect_*_checked (12 call sites); concurrent is the twelfth…") — report labels it optional prose; not required graph hygiene; inventory already complete and status stays done

Residuals not applied (docs/crates/new tickets/authority):
  - none — report lists no required prose, dated correction, remainder filing, or docs/crates edits; Exact files expected change was ticket frontmatter only

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/replace-four-assertions-that-cannot-fail-in-the-cache-and-spike-harnesses/97ef9d1fdf9c_c99ac54950f2.md (full)
    - tickets/replace-four-assertions-that-cannot-fail-in-the-cache-and-spike-harnesses.md (full pre/post)
    - tickets/pin-the-bf16-spike-admitted-operation-verdict-to-its-own-enum.md (exists)
    - tickets/correct-the-accounts-for-every-entry-claim-in-the-cache-research-note-and-harness.md (exists)
  - checks:
    - shasum -a 256 on ticket file after edit
    - related ids match Outcome-named split tickets and exist as ticket files

Recommended next ledger state:
  integrated
