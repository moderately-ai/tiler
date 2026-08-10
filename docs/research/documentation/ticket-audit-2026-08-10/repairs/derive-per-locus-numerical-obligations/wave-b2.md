Ticket: derive-per-locus-numerical-obligations
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/derive-per-locus-numerical-obligations/7826dd08bc14_c99ac54950f2.md
Pre-edit content hash (from ledger): 7826dd08bc147862ce14dc5d34f5351a4e077fba1c03f46842821673d482116f
Post-edit content hash: 0eb41ef80c3191ab3c75f3c41ecaa16132700b5363e87487cb0751749efd82a4

Changes applied:
  - Why single-locus 2026-08-07 Fact: struck as live claim; reframed as historical pre-landing state superseded by Worker record / Integrated (`founded_locus` multi-locus producer at f4901933)
  - Why "What is actually open" … "this ticket is open": struck; Correction — 2026-08-10 that status done and close condition landed multi-locus
  - Worker record rule id: `structure-numerical-realization-locus-weaker-than-ceiling` → `numerical-realization-locus-weaker-than-ceiling` (Structure is ProgramError wrapper, not rule-string prefix)
  - Integrated merge-day artifact-abi / MANIFEST_SCHEMA prose lightly marked merge-day (line :297, agreed-at-merge-day)
  - ## Fact audit — 2026-08-10: (a) multi-locus close at f4901933; (b) MANIFEST_SCHEMA now (16, 0); (c) artifact-abi 15.0 chronological, :297 stale; (d) merge-day pins 23c46a19… / e89c4d82… / 64,542 intermediate rung, live FIXED_CONTENT_BYTES 65_313
  - metadata: none (status done, related, scopes, dependencies left unchanged)

Optional items skipped (with reason):
  - spike README producer-gap paragraph hygiene — report marks optional, out of claimed scopes, already named as dated review record; restated only in Fact audit Remainder so this ticket is not reopened for it
  - related graph edges — report listed none required; existing related set coherent

Residuals not applied (docs/crates/new tickets/authority):
  - none required for this wave; report listed Exact files as ticket only; no new remainder tickets; no docs/crates product edits

Verification:
  - files read: audit report; full ticket pre/post; crates/tiler-artifact/src/program/codec/encode.rs MANIFEST_SCHEMA (16,0); crates/tiler-build/src/metal_plan.rs FIXED_CONTENT_BYTES 65_313 + pin ladder; crates/tiler-compiler/src/session/realization.rs founded_locus path + rule `numerical-realization-locus-weaker-than-ceiling`; docs/artifact-abi.md "moved the manifest schema to 15.0" (near :324)
  - checks: shasum -a 256 post-edit ticket; false present-tense open-work claims struck with 2026-08-10 dated corrections; status/metadata unchanged done

Recommended next ledger state:
  integrated
