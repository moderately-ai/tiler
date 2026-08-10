Ticket: accept-the-proof-bound-stage-coverage-public-boundary
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-proof-bound-stage-coverage-public-boundary/3dea33c2ba34_c99ac54950f2.md
Pre-edit content hash (from ledger): 3dea33c2ba34f61b6db8c4c90c9dac9864cf7ca2fa85a7799fd15c1d4c341249
Post-edit content hash: e26c5e92fa3645cbe1ef1b55e6cd2639fe1727e2f5f7e33651258101a2758c19

Changes applied:
  - Kept `status: done` and other frontmatter metadata (no metadata change required).
  - Added **Correction — 2026-08-10** under Decided — accepted: the "sweep is this record and the catalog of decisions is untouched" claim understated live contract sites; code draft labels remain absent; as of audit base `c99ac54950f2`, docs/ir.md, ADR 0071, and production-crate-codebase-audit.md still deny acceptance in present tense.
  - Expanded implementation/review short hashes to full SHAs (`67765e0068ef5048b9698b4df02ec7b30519f827`, `cd3119f50964af7086f9c93f6e5f4af4181050c4`) in The decision and restated in the correction.
  - Filed remainder ticket `reclassify-the-covered-occurrence-public-boundary-acceptance-labels` (`status: todo`) owning the three-document reclassification; wired `related` on parent and remainder (parent also keeps bind-stage-coverage edge).

Optional items skipped (with reason):
  - none (optional full-SHA expansion applied because hashes resolved cleanly on this host).

Residuals not applied (docs/crates/new tickets/authority):
  - Product edits to `docs/ir.md`, `docs/decisions/0071-use-checked-builders-for-shared-compiler-ir.md`, and `docs/research/documentation/production-crate-codebase-audit.md` remain for the remainder ticket (wave B1 / this wave must not edit those docs here).
  - No crate or rustdoc changes (none required; surface already live).

Verification:
  - files read:
    - tickets/accept-the-proof-bound-stage-coverage-public-boundary.md (full, pre/post)
    - audit report 3dea33c2ba34_c99ac54950f2.md (full)
    - confirmed live "not yet accepted" anchors still present via grep in docs/ir.md, ADR 0071, production-crate-codebase-audit.md
    - house-style samples: accept-the-kernel-program-publishing-copy-surface.md, cite-adr-0095-…, record-the-closure-of-the-quantized-profile-e-1-measurement-gap.md
    - ticketsplease.toml scopes for foundation/decisions/research/documentation
  - checks:
    - `git rev-parse` / `git log -1 --format=%H` for 67765e00 and cd3119f5 → full SHAs above
    - post-edit `shasum -a 256` on the acceptance ticket
    - remainder ticket created with contracts/foundation + contracts/decisions + research/documentation scopes

Recommended next ledger state:
  integrated
