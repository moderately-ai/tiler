Ticket: scope-the-block-scaled-compound-value-vertical
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-block-scaled-compound-value-vertical/a849161bf54a_c99ac54950f2.md
Pre-edit content hash (from ledger): a849161bf54a44f9b5154a81b3b677c86797e07038a67181cf34ea40478eafa0
Post-edit content hash: 5da5c04672d0091906dcf9dc027ca0b854512cd82d8391a2e320271259cb3b10

Changes applied:
  - Required: Trigger check log — left 2026-08-09 historical `todo` line standing; appended 2026-08-10 **not fired** entry stating `implement-workload-selected-quantized-parameter-maps` is `awaiting-decision` (correcting the false status string), PerTensor-only still true, U8 profile still true.
  - Required: User-visible outcome — replaced "one owner" with D-9 / this ticket owns block-scaled compound-value vertical for six OCP MX schemes and *block obligations* of NVFP4/project codecs; *identity* admission remains D-13 (dated Correction — 2026-08-10).
  - Required: Activation trigger — retitled from "in three parts" to two-part MX activation plus separately recorded third reopening route for eliminated affine per-block/per-group maps (dated Correction — 2026-08-10; matches D-9).
  - Metadata: status/priority/dependencies/related/scopes left unchanged (deferred still correct).

Optional items skipped (with reason):
  - none (report optional frontmatter change was "none"; no optional graph hygiene beyond required prose)

Residuals not applied (docs/crates/new tickets/authority):
  - none owed by this repair; no crates/docs/ADR edits; no new remainder tickets; D-13 already owns vendor/project identity

Verification:
  - files read: full audit report; full ticket; implement-workload frontmatter status; ParameterIndexMapKind in types.rs; D-9 trigger and membership table in dtype-family-research-tracks.md
  - checks: implement-workload `status: awaiting-decision`; ParameterIndexMapKind only `PerTensor`; membership row "D-9 for the block obligations, D-13 for identity"; D-9 "Trigger, in two parts"; shasum -a 256 post-edit

Recommended next ledger state:
  integrated
