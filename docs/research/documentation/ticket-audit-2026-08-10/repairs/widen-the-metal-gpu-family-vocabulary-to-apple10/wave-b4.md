Ticket: widen-the-metal-gpu-family-vocabulary-to-apple10
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/widen-the-metal-gpu-family-vocabulary-to-apple10/541ed62b6d55_c99ac54950f2.md
Pre-edit content hash (from ledger): 541ed62b6d55abbfd4759e914a003e21949bae55ef2ea07c94040c660b18a15b
Post-edit content hash: cf6d052d70c0bf590a54738c7367b57d69a178d1c58122a001efdcd89ac496aa

Changes applied:
  - Rephrased the "two records state the range as ending at Apple9" Fact to historical past tense; added **Correction — 2026-08-10** that the research record was fixed to Apple1–Apple10 (`233-242`) by `correct-the-sdk-apple-family-range-in-the-runtime-answer-record`, while close-the-metal-gpu-family Implementation keys still quote the old window and Outcome already records the error.
  - Extended binding-gap / reactivation precondition language from serial-sum-run alone to all metal-0.33.0 nameability asserts, naming `crates/tiler-conformance/src/dispatch.rs` (`MetalGpuFamily::COUNT == 5` + `BINDING_APPLE_FAMILIES`) alongside `prototypes/serial-sum-run/src/proof.rs`, in the binding-gap Fact, "What would close this", Recorded deferral body, and Reactivation triggers.
  - Added **Correction — 2026-08-10 (binding-gap population)** under the recorded deferral documenting the under-stated population.
  - Added 2026-08-10 **not fired** trigger-check log line with reproduce commands for vocabulary, both COUNT sites, and spike measurements.
  - Left status `deferred`, dependencies, related, and scopes unchanged (report: no metadata changes).

Optional items skipped (with reason):
  - Optional related edge for a separate "enumerate metal-binding COUNT couplings" checklist ticket — not mandatory for deferral correctness; no new ticket filed in wave B.
  - Optional related list expansion for tiler-conformance coupling — report said optional for board health; prose already names the site.

Residuals not applied (docs/crates/new tickets/authority):
  - Product reactivation (vocabulary widen, metal binding upgrades, measurement) remains deferred; not product work for this wave.
  - close-the-metal-gpu-family Implementation keys still quote Apple9/`233-241` (owned by that done ticket; Outcome already notes the error); not edited here.
  - crates/tiler-metal MetalGpuFamily still Apple5–Apple9; intentional while deferred.

Verification:
  - files read:
    - full audit report 541ed62b6d55_c99ac54950f2.md
    - full ticket pre- and post-edit
    - docs/research/runtime/backend-scoped-route-requirement-answers.md (rg Apple9/Apple10/233-241/233-242)
    - tickets/close-the-metal-gpu-family-out-of-crate-total-map.md (rg same)
    - crates/tiler-metal/src/applicability.rs (MetalGpuFamily enum)
    - rg MetalGpuFamily::COUNT == 5 / BINDING_APPLE_FAMILIES across prototypes/serial-sum-run and crates/tiler-conformance
  - checks:
    - research record live range is Apple10 / 233-242 (corrected 2026-08-01); close ticket keys still Apple9 / 233-241 with Outcome correction
    - COUNT==5 present in both serial-sum-run proof.rs and tiler-conformance dispatch.rs
    - MetalGpuFamily still Apple5–Apple9 only
    - post-edit shasum -a 256 of ticket file

Recommended next ledger state:
  integrated
