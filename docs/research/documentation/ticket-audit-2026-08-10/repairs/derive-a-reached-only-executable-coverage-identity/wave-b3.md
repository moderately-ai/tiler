Ticket: derive-a-reached-only-executable-coverage-identity
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/derive-a-reached-only-executable-coverage-identity/10ac01e02e02_c99ac54950f2.md
Pre-edit content hash (from ledger): 10ac01e02e025e35930577a30dab86749e26113ad25a8b8234830c5caf6f3881
Post-edit content hash: 88b11d5fc7b80ac1c4e247a640f367e5fb8350c2b45b154145f57ce8688a940f

Changes applied:
  - Retained-subject Fact: added **Correction — 2026-08-10.** that coverage no longer length-frames the full graph identity; live tags are executable-coverage `v2` / staged `v2` opening with a fixed-width digest under `tiler.ir.index-refinement-coverage-graph.v1` (ADR 0104); subject/injectivity and opaque consumer surface unchanged.
  - Omitted-subjects Fact: added **Correction — 2026-08-10.** that live `GRAPH_DOMAIN` is `tiler.semantic-graph.v3`; landing `v2` spelling kept as historical and dated as retired.
  - Closes when: added **Correction — 2026-08-10.** that program/artifact encoder agreement and pin recomputation were owned by `bind-stage-coverage-to-index-refinement-identity` and satisfied when that dependent closed (matches domain-audit ownership).
  - Added `## Fact audit — 2026-08-10` summarizing the three corrections and board leave-as-is.
  - Metadata (status, deps, related, scopes): none required; left unchanged.

Optional items skipped (with reason):
  - none — optional Closes-when clarity was applied (cheap same-ticket hygiene named in Repair required).

Residuals not applied (docs/crates/new tickets/authority):
  - none — report requires ticket-prose only; no docs/crates edits or remainder filing; post-landing domain steps already have owners.

Verification:
  - files read:
    - tickets/derive-a-reached-only-executable-coverage-identity.md (full, pre/post)
    - audit report 10ac01e02e02_c99ac54950f2.md (full)
    - crates/tiler-ir/src/index/refinement.rs (EXECUTABLE_COVERAGE_IDENTITY_TAG v2, COVERAGE_GRAPH_DIGEST_DOMAIN, encode_executable_coverage_identity head digest)
    - crates/tiler-ir/src/semantic/identity.rs (GRAPH_DOMAIN = tiler.semantic-graph.v3)
    - crates/tiler-ir/src/domains.rs (semantic-graph.v3 table row)
  - checks:
    - `EXECUTABLE_COVERAGE_IDENTITY_TAG` = `tiler.ir.index-refinement-executable-coverage.v2\0`
    - `COVERAGE_GRAPH_DIGEST_DOMAIN` = `tiler.ir.index-refinement-coverage-graph.v1\0`; encoder digests `subject.graph.as_bytes()` at head
    - `GRAPH_DOMAIN` = `tiler.semantic-graph.v3\0`
    - no metadata or graph-edge edits

Recommended next ledger state:
  integrated
