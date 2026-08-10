Ticket: carry-a-producer-stated-total-into-a-retained-run
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/carry-a-producer-stated-total-into-a-retained-run/75ff8f7a9892_c99ac54950f2.md
Pre-edit content hash (from ledger): 75ff8f7a9892767c8eb8ed900a834aa0e0d836d49cf2685ca327ba6baffbcaa9
Post-edit content hash: cf340af101733b37304ba79183cb59e17d2914949da8ce12e724ff8a2e944eb0

Changes applied:
  - Why this exists: opening **Correction — 2026-08-10.** records delivery at base `c99ac54950f2` / landing `c39cb814` — `retaining_with_stated_total` exists, `stage_retention` states `total_bytes()`, gap section replaced by "**The stage's own total is stated, not re-derived.**"; Facts labelled filing-time only
  - Fact 1 title reframed as pre-fix failure mode of `retaining` alone; body notes length-derived path still hides truncation while product path states the producer total
  - Fact 2 struck to historical-at-filing form; present-tense "cannot state it" / "no parameter" / "doc names this gap" closed with inline **Correction — 2026-08-10.** pointing at `retaining_with_stated_total(..., output.total_bytes())` and the rewritten Metal module doc
  - Metadata left unchanged (status done, deps [], related, scopes stand)

Optional items skipped (with reason):
  - optional brief Outcome: report listed it as optional documentary thinness only; close condition already evidenced by source and the dated correction; not required for repair completeness

Residuals not applied (docs/crates/new tickets/authority):
  - none; report required no docs/crates edits and no new remainder tickets (producer-side pairing already split to `make-stage-retention-reachable-from-a-test`, done)

Verification:
  - files read: full audit report; full ticket pre-edit; greps on `crates/tiler-build/src/metal_cache.rs` (`retaining_with_stated_total`, `total_bytes`, "**The stage's own total is stated, not re-derived.**"), `crates/tiler-cache/src/expansion/retention.rs` (`retaining` → `retaining_with_stated_total`), `crates/tiler-cache/src/expansion/tests.rs` (`a_producer_stated_total_survives_to_a_validated_hit`), `MAX_RETAINED_*` constants
  - checks: shasum -a 256 post-edit ticket; present-tense Fact 2 claims no longer readable as live inventory; metadata unaltered

Recommended next ledger state:
  integrated
