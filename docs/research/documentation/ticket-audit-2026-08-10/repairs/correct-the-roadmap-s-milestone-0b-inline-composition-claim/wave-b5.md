Ticket: correct-the-roadmap-s-milestone-0b-inline-composition-claim
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/correct-the-roadmap-s-milestone-0b-inline-composition-claim/8e6b7663818d_c99ac54950f2.md
Pre-edit content hash (from ledger): 8e6b7663818d529651f74b4b0a7b02239f38348f6249455ba5767cb46b2abef4
Post-edit content hash: a0288b335ab11c79da07340358db7e726e1eb7186cc0ef04268601ac2ecd0982

Changes applied:
  - Frontmatter `related`: [] → [correct-the-stale-fallbackonly-claims-in-tiler-macros-family-cfg] (optional graph hygiene; Outcome filed that sibling; sibling is itself done).
  - Appended `## Fact audit — 2026-08-10`: close condition still met / status remains done; sibling family_cfg Outcome "Filed, not fixed" marked landing-time (sibling done; false FallbackOnly claims gone from family_cfg.rs); residual false live status.md roadmap-pointer recorded with reproduce command.
  - Metadata status/deps/scopes left unchanged (report: none required).

Optional items skipped (with reason):
  - none (optional related edge applied).

Residuals not applied (docs/crates/new tickets/authority):
  - Required dated correction in `docs/status.md` under the inline developer experience Milestone 0B paragraph: live sentence "That milestone's own text still asserts that inline composition does not exist" is false after this ticket landed; replace or date-correct so it records this ticket landed and the roadmap no longer asserts composition absence (keep consumer-integration and second-family absences; keep non-judgement of milestone exit). Class C ticket-only wave — not edited. Reproduce: `rg -n 'still asserts that inline composition' docs/status.md`.
  - No new remainder ticket filed (report: optional related only; no unsplit implementation remainder for this ticket's own scope).
  - Outcome merge/worker SHAs (`388f07ba` / `643caeeb`) not re-validated with git show (report residual uncertainty only).

Verification:
  - files read:
    - full audit report 8e6b7663818d_c99ac54950f2.md
    - full ticket correct-the-roadmap-s-milestone-0b-inline-composition-claim.md (pre + post)
    - docs/status.md (rg residual "still asserts that inline composition")
    - tickets/correct-the-stale-fallbackonly-claims-in-tiler-macros-family-cfg.md frontmatter (status done; related [])
  - checks:
    - status.md residual still present at current tree
    - post-edit `shasum -a 256` → a0288b335ab11c79da07340358db7e726e1eb7186cc0ef04268601ac2ecd0982

Recommended next ledger state:
  integrated
