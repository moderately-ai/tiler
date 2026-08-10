Ticket: decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode/8c0cc1875b81_c99ac54950f2.md
Pre-edit content hash (from ledger): 8c0cc1875b81d3736ec8ae221caf919a4a5f8ee6d5be785717e5cfe2fdb9e946
Post-edit content hash: 1e060d156c06ec95b00217ed3ad940e61b801a24172c4f901f71bbdbb550519e

Changes applied:
  - Outcome identity Fact: replaced stale ordinals `identity.rs:87` / lines 103 and 125 / `shape.rs:60` with searchable anchors (`fn compute_graph_identity`, the two `shape.encode(&mut bytes)` sites, `pub struct Extent`); thirteen-identity inference unchanged.
  - Outcome numerical Measurement: replaced sole `decoder_layer.rs:1417` with end-to-end test anchors `the_layer_evaluates_end_to_end_at_the_c1_prefill_row` / `the_layer_evaluates_end_to_end_at_the_c1_decode_row` and `differing(..., 0)`.
  - Graph maintenance symbolic-extents census: reframed as 2026-08-05 snapshot (0/0/55) with 2026-08-10 recheck (broadcast 2, mapping 0, extent 65); directional gap claim kept.
  - Graph maintenance child ticket: filing-day `deferred` / deps `todo` marked historical; dated note that child is `awaiting-decision` after deps closed and trigger fired 2026-08-09; Inference past-tense adjusted to match.
  - One **Correction — 2026-08-10.** block after Measurement boundary covering items 1–4; elimination table and candidate 7 decision untouched.
  - Metadata: no changes (status done; scopes; related edges already correct).

Optional items skipped (with reason):
  - none (optional child-status note applied as cheap graph hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - L6 (`docs/research/program-planning/complete-model-ingestion-and-execution.md`) still says the define-widening ticket is `deferred` in places while the ticket is `awaiting-decision` — out of ticket ownership; separate record repair if a sweep owns L6.
  - No new remainder tickets; child already related and correctly edged for D-19.

Verification:
  - files read:
    - tickets/decide-whether-one-decoder-layer-graph-can-serve-prefill-and-decode.md (full, before and after)
    - audit report 8c0cc1875b81_c99ac54950f2.md (full)
    - tickets/define-the-widening-relation-over-a-symbolic-broadcast-extent.md frontmatter (status awaiting-decision)
    - crates/tiler-ir/src/semantic/identity.rs anchors via rg (`fn compute_graph_identity`, two `shape.encode`)
    - crates/tiler-ir/src/shape.rs (`pub struct Extent`)
    - crates/tiler-reference/tests/decoder_layer.rs end-to-end test names / differing asserts
    - docs/research/shapes/symbolic-semantic-extents.md greps: broadcast 2, mapping 0, extent 65
  - checks:
    - live Outcome no longer cites identity.rs:87 / shape.rs:60 / decoder_layer.rs:1417 as present locations
    - correction block present; elimination table unrewritten
    - post-edit sha256 computed on ticket file

Recommended next ledger state:
  integrated
