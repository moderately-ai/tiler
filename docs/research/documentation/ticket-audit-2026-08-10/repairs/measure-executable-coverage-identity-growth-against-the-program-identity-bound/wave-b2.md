Ticket: measure-executable-coverage-identity-growth-against-the-program-identity-bound
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/measure-executable-coverage-identity-growth-against-the-program-identity-bound/94fc6ffa6213_c99ac54950f2.md
Pre-edit content hash (from ledger): 94fc6ffa621346242ce185adcdf844c4835b30dbf00a576f3a51b4cfaf92f229
Post-edit content hash: e28cb3e424f1fe1ea925f0f6d503263bd743ddf382cf3a2554f6aa5b3b433447

Changes applied:
  - related: added decide-whether-executable-coverage-evidence-folds-as-a-digest, add-the-identity-growth-experiment-rows-to-the-two-catalogs, widen-the-identity-growth-ladder-to-the-governed-operation-budget, rebaseline-the-identity-growth-ladder-on-the-derived-region-shape-budgets (bind-stage parent kept); status stays done
  - Why this exists: dated Correction — 2026-08-10 stating live digest fold + linear fit; filing-time quadratic embedding retained as historical motivation; rotten line citations replaced by searchable anchors (pub struct SemanticGraphIdentity(Vec<u8>), fn encode_executable_coverage_identity, MAX_PROGRAM_IDENTITY_BYTES = 64 * 1024 * 1024 / pub const MAX_PROGRAM_IDENTITY_BYTES)
  - What this ticket owes: digest contingency softened to past tense / discharged pointing at ADR 0104; structural redundancy restated as historical fact the ADR rested on rather than an open design question
  - Outcome: first delivery at 5568bf19 / pre-fold fit and margin; ADR 0104 accepted 2026-08-06; latest ladder results/2026-08-08-post-sourced-semantic-shape-… and linear fit 3531n + 724 with labelled refusal n=19006 and ×371 margin

Optional items skipped (with reason):
  - none (optional related-list graph hygiene applied in full for reciprocal discoverability)

Residuals not applied (docs/crates/new tickets/authority):
  - none (Exact files listed only this ticket; no docs/crates remainder; no new remainder tickets required)

Verification:
  - files read: audit report; ticket (pre-edit); crates/tiler-ir/src/semantic/identity.rs (SemanticGraphIdentity); crates/tiler-ir/src/index/refinement.rs (encode_executable_coverage_identity + COVERAGE_GRAPH_DIGEST_DOMAIN digest path); crates/tiler-ir/src/program/mod.rs (MAX_PROGRAM_IDENTITY_BYTES); docs/decisions/0104-fold-the-per-record-graph-identity-as-a-digest.md (accepted/implemented + 3531n+724 ladder); spikes/program-planning/identity-growth/README.md (ticket id, fit, verdict, margin); spikes/…/results/ listing (2026-08-08-post-sourced-semantic-shape-… present); related frontmatter on decide/catalog/widen/rebaseline tickets
  - checks: greps for anchors hit; shasum -a 256 of ticket post-edit = e28cb3e424f1fe1ea925f0f6d503263bd743ddf382cf3a2554f6aa5b3b433447

Recommended next ledger state:
  integrated
