Ticket: bind-repeated-invocations-over-caller-retained-tensors
Wave: B5
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/bind-repeated-invocations-over-caller-retained-tensors/e1e32eeca509_c99ac54950f2.md
Pre-edit content hash (from ledger): e1e32eeca509187289d901c582f9fd186b2d0ccdde358bac1ed5abca2ad16096
Post-edit content hash: cc8e49a84747f89b2406204a1f023a46d3a60dc1224d8027af1eb8e336c74ba6

Changes applied:
  - dependencies: removed `admit-the-sequence-extension-concatenate-family` (no consumption site after generic rewrite); related: added that id at the front of the list.
  - Supersession rationale: softened "wrong key" — positional `bind_region` refuses count/rank/stored scalar/literal extent; keyed `RegionRequest` / `BindingTarget::ProgramInput` paths named separately.
  - Required behaviour longer-resource bullet: named facade/adapter split — adapter `plan_dispatch` allows longer storage; `tiler` `checked_length` / `BindError::StorageLengthMismatch` requires equality and must be relaxed or bypassed on the path this ticket owns (`reach ≤ storage`); Tom-owned if it widens bind contracts.
  - Required behaviour multi-variant bullet: replaced "Package the tiled realization … and the direct realization" with two complete variants (fixture or already-landed) under `RoutingPolicy::StablePriority` and extent `≡ 0 (mod 16)`, without requiring deferred `realize-the-tiled-contraction-schedule-and-its-metal-emission`; noted metal-scope exclusion.
  - Closes when: same synthetic multi-variant sufficiency and longer-resource facade obligation; no deferred Metal tiled body as close criterion.
  - `## Fact audit — 2026-08-10` with four dated correction blocks and reproduce anchors.

Optional items skipped (with reason):
  - none; optional dated correction block applied as house-style hygiene on the same ticket.

Residuals not applied (docs/crates/new tickets/authority):
  - crates/tiler/src/route.rs and value.rs — StorageLengthMismatch equality policy (product; Class E ticket-only).
  - crates/tiler-artifact — specialization-on-bound-extent assembly refusal and multi-variant packaging tests (product).
  - crates/tiler-runtime and tests — multi-extent rebinding, identity pin, live-span oracle (product).
  - crates/tiler-build tests as needed for assembly diagnostics (product).
  - Live-extent prerequisite and symbolic-extent chain remain undelivered; readiness still blocked on `admit-live-extent-operands-to-payload-indexing`.
  - No new remainder ticket: preferred repair uses synthetic multi-variant sufficiency; real tiled chain not connected as a hard dep.

Verification:
  - files read: full audit report; full ticket (pre/post); crates/tiler/src/route.rs (checked_length / declared == actual); crates/tiler/src/value.rs (BindError population); crates/tiler-runtime/tests/adapter_route/adapter.rs (supplied < reach); crates/tiler-runtime/src/load.rs (StablePriority select_variant); tickets/realize-the-tiled-contraction-schedule-and-its-metal-emission.md (status: deferred).
  - checks: shasum -a 256 post-edit; rg anchors for dependencies/related/two complete variants/StorageLengthMismatch/Fact audit; linked ticket ids exist on disk.

Recommended next ledger state:
  integrated
