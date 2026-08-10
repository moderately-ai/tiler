Ticket: wire-the-env-configured-eviction-policy-through-the-deliver-path
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/wire-the-env-configured-eviction-policy-through-the-deliver-path/da3e88ec9c2e_c99ac54950f2.md
Pre-edit content hash (from ledger): da3e88ec9c2e662ad3adedd2357cd3d1537c70d07ece724e5cedc8079bad30aa
Post-edit content hash: 481cffe2d3d1cef7b19458de1fcba69c4b8b79861e3e45c45935304c5eb8b014

Changes applied:
  - Added **Correction — 2026-08-10.** under `### What the contract still owes` stating that the Compiler cache section of `docs/integration/frontends.md` already documents variable, default (`MaxEntryAge::DEFAULT` / 30d), opt-out `off`, spellings, refusal behaviour, trigger, amortization, and report disposition; the "live sibling / Integrator edits paste" residual is historical, not live debt.
  - Metadata left unchanged (status, deps, related, scopes) per report.

Optional items skipped (with reason):
  - Tighten "Found while implementing" collect.rs path to expansion.rs and "off the hit path" wording — skipped after re-read. `crates/tiler-cache/src/expansion/collect.rs` still states the automatic caller "invokes this operation off / the hit path" (line-broken `//!` docs); the ticket citation is accurate. Parent `expansion.rs` has related but different wording (`configured through environment variables and off the hit path`). Applying the optional rewrite would introduce a false claim.

Residuals not applied (docs/crates/new tickets/authority):
  - none required by report (no docs/crates edits; no new remainder; size ceilings already deferred on `configure-a-size-ceiling-for-the-automatic-expansion-cache-eviction`).

Verification:
  - files read:
    - tickets/wire-the-env-configured-eviction-policy-through-the-deliver-path.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/wire-the-env-configured-eviction-policy-through-the-deliver-path/da3e88ec9c2e_c99ac54950f2.md
    - docs/integration/frontends.md (Compiler cache / eviction paragraphs)
    - crates/tiler-cache/src/expansion.rs (module docs, "off the hit path")
    - crates/tiler-cache/src/expansion/collect.rs (module docs, "invokes this operation off the hit path")
  - checks:
    - frontends.md documents `TILER_EXPANSION_CACHE_MAX_ENTRY_AGE`, MaxEntryAge::DEFAULT / thirty days, `off`, spellings, refusal, post-publication trigger, amortization, report non-surfacing (anchors `The cache trims itself`, `An expansion reads one further variable`)
    - collect.rs still carries the ticket's quoted claim across two `//!` lines; optional path retarget declined
    - post-edit sha256: 481cffe2d3d1cef7b19458de1fcba69c4b8b79861e3e45c45935304c5eb8b014

Recommended next ledger state:
  integrated
