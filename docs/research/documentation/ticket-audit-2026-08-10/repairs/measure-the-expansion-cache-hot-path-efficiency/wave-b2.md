Ticket: measure-the-expansion-cache-hot-path-efficiency
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/measure-the-expansion-cache-hot-path-efficiency/4c6f1791521a_c99ac54950f2.md
Pre-edit content hash (from ledger): 4c6f1791521a7080db38acb1b444063d2ea2dd427fd9c28a259ce8be3f126815
Post-edit content hash: a19686962ce9fe677549feba1bbd043eb5470ea7f1a3da22e822d2c28911528c

Changes applied:
  - Expanded `related:` with decide-whether-the-bundle-envelope-section-digest-is-redundant, restore-the-cache-build-tool-exercise-against-the-current-artifact-api, catalog-the-cache-hot-path-efficiency-records, re-derive-the-measured-envelope-band-the-cache-hot-path-sweeps, re-price-the-envelope-band-consumers-against-the-re-derived-band, wire-the-env-configured-eviction-policy-through-the-deliver-path (navigability only; status/deps/scopes unchanged).
  - Added `**Correction — 2026-08-10.**` after Outcome: four retained TSV pairs (2026-08-04 at 32,136–47,803 B and 2026-08-06 at 141,532–159,037 B); Outcome bullets remain first-campaign record (not rewritten); current-band figures in research note Section 9 / 9.7; follow-ons wire-the-env, digest decision, restore, catalog, re-derive, and re-price all since-done; catalogs live in docs/research/README.md and spikes/README.md; close condition and status: done still hold.

Optional items skipped (with reason):
  - none (related expansion applied as cheap graph hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - none. Report required ticket-only dated correction; no docs/crates edits or new remainder tickets.

Verification:
  - files read:
    - tickets/measure-the-expansion-cache-hot-path-efficiency.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/measure-the-expansion-cache-hot-path-efficiency/4c6f1791521a_c99ac54950f2.md
    - spikes/cache/hot-path-efficiency/results/ (four TSV names)
    - spikes/cache/hot-path-efficiency/harness/src/main.rs (SIZES = 141_532, 159_037)
    - docs/research/cache/hot-path-efficiency.md (Section 9 / 9.7 anchors, research_status complete)
    - status frontmatter of the six follow-on tickets named in the correction (all done)
  - checks:
    - ls results/ → 4 files
    - rg '^status:' on each follow-on → done
    - sha256sum post-edit ticket → a19686962ce9fe677549feba1bbd043eb5470ea7f1a3da22e822d2c28911528c

Recommended next ledger state:
  integrated
