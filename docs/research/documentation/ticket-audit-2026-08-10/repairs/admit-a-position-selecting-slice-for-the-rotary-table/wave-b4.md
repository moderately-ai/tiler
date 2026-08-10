Ticket: admit-a-position-selecting-slice-for-the-rotary-table
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-position-selecting-slice-for-the-rotary-table/1b096c4cf74d_c99ac54950f2.md
Pre-edit content hash (from ledger): 1b096c4cf74dde476599d286b334799234df8ac8642eb9ad0f23b4ee8b2a948c
Post-edit content hash: 8af23016f719c18d77711991a0ebba945ba50b1f6c8421201e7b23d73e5bf6f5

Changes applied:
  - scopes: dropped exclusive `contracts/foundation`; set exclusive `contracts/navigation` and `shared_scopes: [project/tickets]` (compose-rotary pattern).
  - "Why this is a correctness trigger": removed "about" before the L6 byte pair; wording is now "4,978,634,752 bytes against 607,744" (matches L6 absolute digits as the repair required).
  - Same paragraph: dropped unanchored "with a contiguous cache" on the absolute-position claim; retained "at batch 1" and the L5 Inference wording.

Optional items skipped (with reason):
  - none (no optional bullets beyond the preferred scopes pattern already applied).

Residuals not applied (docs/crates/new tickets/authority):
  - Edge implementation carrier as hard dependency after decide-the-source-bearing-slice-offset-boundary lands and names/files that carrier (report: expected interim incompleteness; not edged yet).
  - Product delivery (source-bearing slice consumer program, matrix evidence) remains open; status stays `todo`.
  - Re-verify note: `8192*151936*4` is 4,978,638,848 on this host, so the L6-cited 4,978,634,752 figure is off by 4,096; this ticket still quotes L6 per the repair required (authority for correcting the absolute is L6 / project-only-the-final-position-logits, not this consumer ticket).

Verification:
  - files read:
    - tickets/admit-a-position-selecting-slice-for-the-rotary-table.md (pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-position-selecting-slice-for-the-rotary-table/1b096c4cf74d_c99ac54950f2.md
    - tickets/compose-rotary-position-embedding-from-reindex-and-broadcast.md (scopes pattern)
    - docs/research/runtime/autoregressive-state-and-kv-cache.md (absolute-position Inference anchor)
    - docs/research/program-planning/complete-model-ingestion-and-execution.md (L6 byte pair wording)
  - checks:
    - python3 -c 'print(8192*151936*4, 151936*4)' → 4978638848 607744
    - shasum -a 256 tickets/admit-a-position-selecting-slice-for-the-rotary-table.md → 8af23016f719c18d77711991a0ebba945ba50b1f6c8421201e7b23d73e5bf6f5
    - status left `todo` (report: correct)

Recommended next ledger state:
  integrated
