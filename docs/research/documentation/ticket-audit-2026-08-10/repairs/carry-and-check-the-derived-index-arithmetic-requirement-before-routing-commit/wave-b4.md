Ticket: carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit/24165bf24a73_c99ac54950f2.md
Pre-edit content hash (from ledger): 24165bf24a737c038b0c8fe1ac2d20b90b90edcdc4bcb88345469a6b9261aa57
Post-edit content hash: bba9b3e708139373193be0aa4b37ba13f72cbd135b555c72a0ac302391eadbdb

Changes applied:
  - Marked **Fact — construction** as historical pre-landing shape; added **Correction — 2026-08-10** stating live `index_arithmetic_requirement(requirements.index_arithmetic)` classification (no KernelType re-derive) and emission anchors (`KernelType::Index => Ok("uint64_t")`, structured-index comment).
  - Marked **The missing mechanism** producer-loss claim as historical pre-landing gap; added **Correction — 2026-08-10** that `pub index_arithmetic: IndexArithmetic` is carried, derived via `REGION_INDEX_ARITHMETIC`, encoded via `push_resources`, published by `DecodedEntry::resources()`; Outcome remains delivered truth.
  - Softened pre-landing backend comparison prose as pre-landing observation (discharge under Outcome).
  - Replaced stale line-number-first authority/ownership citations with searchable anchors (`CompleteU64` ledger section; `mints no route row` / live-device derivability prose).
  - Outcome unmoved pin list: dated explain qualifier `940c09e0821665a6` as landing-time; noted later rebaseline to live `7ba3d77a66f04638`.
  - Draft public boundary: `IndexArithmetic` type/field in schedule; `IndexArithmetic::of` impl placement in kernel model.
  - Optional graph hygiene: reframed `remove-the-workload-shapes…` forward rebaseline note as historical; **Correction — 2026-08-10** that ticket is `done`.
  - Status/dependencies/related/scopes left unchanged (`awaiting-decision` remains correct for Tom acceptance of draft surface).

Optional items skipped (with reason):
  - none (optional workload-shapes reframe applied as cheap same-ticket hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - Tom acceptance of the draft public surface (Closes when; not wave-B product work).
  - Live Apple9 proof-matrix re-measurement (audit residual; not ticket-prose debt).
  - No crates/docs edits; no new remainder tickets required by the report.

Verification:
  - files read:
    - tickets/carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit.md (full, pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/carry-and-check-the-derived-index-arithmetic-requirement-before-routing-commit/24165bf24a73_c99ac54950f2.md (full)
    - crates/tiler-compiler/src/physical.rs (region_proposal / index_arithmetic_requirement anchors via rg)
    - crates/tiler-ir/src/schedule/model.rs (`pub index_arithmetic`, REGION_INDEX_ARITHMETIC)
    - crates/tiler-ir/src/kernel/model.rs (`impl IndexArithmetic` / `of`)
    - crates/tiler-artifact/src/program (push_resources, resources)
    - crates/tiler-compiler/src/explain.rs (live explain pin `7ba3d77a66f04638`)
    - tickets/remove-the-workload-shapes-from-the-concatenate-normative-definition.md (status: done)
  - checks:
    - `rg` confirmed no live `index_arithmetic_requirement(KernelType::Index)`; live form is `index_arithmetic_requirement(requirements.index_arithmetic)`
    - explain pin `tiler-explain-v7 request=7ba3d77a66f04638`
    - shasum -a 256 post-edit ticket → bba9b3e708139373193be0aa4b37ba13f72cbd135b555c72a0ac302391eadbdb

Recommended next ledger state:
  integrated
