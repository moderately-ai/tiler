Ticket: admit-an-age-bounded-automatic-eviction-into-the-expansion-cache
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-an-age-bounded-automatic-eviction-into-the-expansion-cache/d5ffe21f5fcd_c99ac54950f2.md
Pre-edit content hash (from ledger): d5ffe21f5fcdee07f50044b166714ca28ad14115261e776a4eb24ad1f213150f
Post-edit content hash: d5ffe21f5fcdee07f50044b166714ca28ad14115261e776a4eb24ad1f213150f

Changes applied:
  - none (report: exact metadata none; ticket body none required — terminal Acceptance already states the age vocabulary is accepted as built; intermediate Implementation narrative may keep pre-acceptance draft language under terminal-record convention; status `done`, deps `[]`, related triple, scopes stay)

Optional items skipped (with reason):
  - Optional one-line dated acceptance note in `docs/research/cache/bounded-collection.md` withdrawing the "does not settle" draft claim — corpus path, Class E forbids docs edits in this wave; listed under Residuals with the required corpus rewrite.
  - Optional graph hygiene for `re-price-the-envelope-band-consumers-against-the-re-derived-band` on `related` — not listed under Repair required; DEFAULT ground re-price is already owned by that done ticket and live DEFAULT docs; no discoverability defect on this ticket's Closes-when.

Residuals not applied (docs/crates/new tickets/authority):
  - product residual (corpus, Class E no crates/docs edit): `crates/tiler-cache/src/expansion.rs` module docs still say the age vocabulary "is a reviewed *draft* under ADR 0074 convention 7 and is not yet accepted" — false after Acceptance 2026-08-04 / collect.rs `# Boundary status` ("were accepted on 2026-08-04 as the age extension…"). Replace with language matching collect.rs Boundary status (accepted under orchestrator delegation / ticket Acceptance).
  - product residual (corpus): `docs/research/cache/bounded-collection.md` *What this design does not settle* public-facade bullet still claims the age vocabulary is "a draft, not an acceptance" and items "await Tom's ruling" — false; rewrite or remove that claim for `MaxEntryAge` / `MaxEntryAgeRefusal` / `RemovalReason` / `CollectionBound::max_entry_age` / `RemovedEntry::reason`. Prefer a dated 2026-08-04 acceptance note over silent rewrite of supersession history. Status-line pointer above *Who runs it* that still says "age vocabulary that is still a draft on it" needs the same alignment.
  - no new remainder ticket: report says documentation alignment is a small corpus fix, not a new capability; mechanism Closes-when met; no unsplit live remainder of the original Closes-when.

Verification:
  - files read:
    - entire audit report `…/d5ffe21f5fcd_c99ac54950f2.md`
    - entire ticket `tickets/admit-an-age-bounded-automatic-eviction-into-the-expansion-cache.md`
    - `crates/tiler-cache/src/expansion.rs` module docs around age-vocabulary draft claim
    - `crates/tiler-cache/src/expansion/collect.rs` `# Boundary status` (accepted 2026-08-04)
    - `docs/research/cache/bounded-collection.md` status line, supersession, *What this design does not settle* public-facade bullet
  - checks:
    - ticket sha256 unchanged: `d5ffe21f5fcdee07f50044b166714ca28ad14115261e776a4eb24ad1f213150f` (matches ledger)
    - residual anchors still live: `vocabulary is a reviewed *draft*` in expansion.rs; `age vocabulary added to that facade is a draft, not an acceptance` in bounded-collection.md
    - authority: collect.rs `were accepted on 2026-08-04 as the age extension`; ticket Acceptance section matches
    - frontmatter status `done`; related decide/wire/measure unchanged; no mechanism remainder required

Recommended next ledger state:
  integrated
