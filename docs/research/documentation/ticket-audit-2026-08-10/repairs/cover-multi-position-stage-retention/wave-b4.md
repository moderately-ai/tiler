Ticket: cover-multi-position-stage-retention
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/cover-multi-position-stage-retention/1e26d82c0631_c99ac54950f2.md
Pre-edit content hash (from ledger): 1e26d82c0631332c88a0dc8bdd1abbfcc5c6e025289abe7f1ff0c541bb2cc417
Post-edit content hash: 90e89d948294ef9bdcb307253924c748f8012df7b97daf6a8e4e7b0d2d47e366

Changes applied:
  - status deferred → todo (two-position trigger fired; close condition unmet).
  - related: added carry-one-payload-per-artifact-family-in-one-envelope (supplier of second_artifact_family_fixture and two-position plan path; status done).
  - Retitled deferral section to "Why this is open rather than deferred"; **Correction — 2026-08-10** records trigger fire, false pub-fn-only census, remaining multi-position retention gap, and optional nine-position elision half.
  - Softened opening Fact: public/production still one constructor and one-element slices; live cfg(test) second constructor makes position 1 constructible; multi-position *retention* (not constructibility) is the gap.
  - Position-0 Fact kept (verified); noted no tiler.metal.1.* retention asserts.
  - Retired Inference that multi-position coverage needs a public StageOutputs surface; remaining work is second_artifact_family_fixture + warning_toolchain through accept_or_publish_metal_plan.
  - Trigger prose: test-only half fired; product multi-family delivery not required for the two-position retention test.
  - Trigger check log: 2026-08-10 **fired** with second_artifact_family_fixture + two-family accept_or_publish_metal_plan anchors; notes 2026-08-09 pub-fn-only miss; elision half still not product-reachable.

Optional items skipped (with reason):
  - none (optional related carry-one-payload… applied as cheap graph hygiene).

Residuals not applied (docs/crates/new tickets/authority):
  - crates/tiler-build/src/metal_plan.rs multi-position retention test(s) — product work outside wave-B ticket-only repair; remains this ticket's close condition.
  - Nine-position elided_retention coverage — Closes when already permits leaving deferred in-ticket; no remainder ticket filed.

Verification:
  - files read:
    - tickets/cover-multi-position-stage-retention.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/cover-multi-position-stage-retention/1e26d82c0631_c99ac54950f2.md (full)
    - crates/tiler-build/src/metal_declaration.rs (first_macos_apple9; cfg(test) second_artifact_family_fixture)
    - crates/tiler-build/src/metal_plan.rs (position-0 retention labels; two-family delivery_positions == 2)
    - crates/tiler-build/src/metal_cache.rs (stage_retention / stage_label / elided_retention anchors via search)
    - crates/tiler-cache/src/expansion/retention.rs (MAX_RETAINED_RUNS = 16 via search)
    - tickets/carry-one-payload-per-artifact-family-in-one-envelope.md (status: done)
  - checks:
    - rg second_artifact_family_fixture in metal_declaration.rs + metal_plan.rs
    - rg 'delivery_positions(), 2' in metal_plan.rs (three asserts)
    - rg tiler.metal.1.(metal|metallib) under tiler-build (no retention hit)
    - shasum -a 256 post-edit: 90e89d948294ef9bdcb307253924c748f8012df7b97daf6a8e4e7b0d2d47e366

Recommended next ledger state:
  integrated
