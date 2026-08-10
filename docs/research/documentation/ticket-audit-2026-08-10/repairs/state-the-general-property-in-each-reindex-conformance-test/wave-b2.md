Ticket: state-the-general-property-in-each-reindex-conformance-test
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/state-the-general-property-in-each-reindex-conformance-test/011f90659880_c99ac54950f2.md
Pre-edit content hash (from ledger): 011f90659880fd97e3cce896c446aa7d85bc548b9f6db5dbd7b7fe6c42c288a5
Post-edit content hash: b66976816bfe495ee46cb6055d8389711e12ee9c8944cb04016a8a79f56c6946

Changes applied:
  - Non-goals: replaced rotted `roadmap.md:374` line citation with searchable anchor to the consumer-conformance-fixture classification phrase `may name the workload freely`.
  - Outcome serial_sum_slice residual: past-tensed exclusion as delivery-time (`undocumented at delivery`); dropped live "remains" inventory voice.
  - Added **Correction — 2026-08-10.** naming `df1219d7` and the property-first doc opener that closed the residual the same day.

Optional items skipped (with reason):
  - none (report listed no optional repair bullets).

Residuals not applied (docs/crates/new tickets/authority):
  - none — required items were ticket prose only; no remainder ticket, docs/, or crates/ edits demanded.

Verification:
  - files read:
    - tickets/state-the-general-property-in-each-reindex-conformance-test.md (full, pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/state-the-general-property-in-each-reindex-conformance-test/011f90659880_c99ac54950f2.md (full)
    - docs/roadmap.md (anchor `may name the workload freely` at classification table row for consumer conformance fixture)
    - crates/tiler-reference/tests/serial_sum_slice.rs (module header + awk undoc census)
    - git show df1219d7 (message + 10-line /// insertion)
  - checks:
    - `rg 'may name the workload freely' docs/roadmap.md` → 1 hit in class-conformance-fixture cell
    - awk undoc on serial_sum_slice.rs at HEAD → empty (documented)
    - `git log -1 df1219d7` → "Document the general property in the serial-sum-slice proof test"
    - shasum -a 256 on ticket after edit → b66976816bfe495ee46cb6055d8389711e12ee9c8944cb04016a8a79f56c6946

Recommended next ledger state:
  integrated
