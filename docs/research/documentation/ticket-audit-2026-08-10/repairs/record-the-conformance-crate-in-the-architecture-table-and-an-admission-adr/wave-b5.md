Ticket: record-the-conformance-crate-in-the-architecture-table-and-an-admission-adr
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/record-the-conformance-crate-in-the-architecture-table-and-an-admission-adr/a26f988743b1_c99ac54950f2.md
Pre-edit content hash (from ledger): a26f988743b11d83414e1ac5671bf74d1481ab28e9ea2cf724f92e34841ec6fd
Post-edit content hash: cee9a674a4c05865070cd0c90fd9d85f93607d8310be573923373d179459bec0

Changes applied:
  - Rewrote the opening of `## Current-state audit — 2026-08-09` so present-tense currency claims cover component-ownership anti-goals and empty reverse-dependent design only (not the packaging edge list).
  - Added `**Correction — 2026-08-10.**` naming the false 2026-08-09 packaging/dependency currency claims, commit `082ad4b9`, produce-envelope ticket, direct `tiler-cache` edge vs still-transitive `tiler-digest`, and the residual architecture + ADR 0106 item 2 supersession work.

Optional items skipped (with reason):
  - optional `related[]` expansions (produce-envelope, survey, refresh, payload-limit, etc.): report labeled optional hygiene only; terminal admission ticket already cites them in prose; no depends-on change.

Residuals not applied (docs/crates/new tickets/authority):
  - `docs/architecture.md`: packaging block `tiler-conformance -> [...]` must include `tiler-cache`; rewrite the paragraph claiming tiler-cache is only transitive (cite publication path / produce-envelope Outcome).
  - `docs/decisions/0106-admit-tiler-conformance-as-the-cross-layer-evidence-member.md`: dated supersession beside Decision item 2 "tiler-cache and tiler-digest are reached transitively and deliberately not named" (cache no longer purely transitive; design rule still holds for digest); do not rewrite admission-era Decision as original.
  - No new remainder ticket filed this wave (Class C ticket-only default; produce-envelope owns the edge, not the architecture prose).

Verification:
  - files read:
    - full audit report a26f988743b1_c99ac54950f2.md
    - full ticket (pre/post edit)
    - crates/tiler-conformance/Cargo.toml (tiler-cache.workspace + publication rationale)
    - docs/architecture.md anchors for packaging row and "deliberately not named"
    - docs/decisions/0106 item 2 transitive sentence
    - produce-the-conformance-envelope frontmatter (status done)
  - checks:
    - rg tiler-cache.workspace in tiler-conformance Cargo.toml: direct edge present
    - architecture packaging line still omits tiler-cache
    - post-edit shasum -a 256 = cee9a674a4c05865070cd0c90fd9d85f93607d8310be573923373d179459bec0

Recommended next ledger state:
  integrated
