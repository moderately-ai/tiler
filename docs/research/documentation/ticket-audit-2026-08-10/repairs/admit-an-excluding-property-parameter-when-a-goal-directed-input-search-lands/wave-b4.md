Ticket: admit-an-excluding-property-parameter-when-a-goal-directed-input-search-lands
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/admit-an-excluding-property-parameter-when-a-goal-directed-input-search-lands/45645b4786f0_c99ac54950f2.md
Pre-edit content hash (from ledger): 45645b4786f03e5db83cac0f1e9a4c1620e424031332cdd7166ddb4e34568e58
Post-edit content hash: f1e0f4931cd6fa80020ad51091710146bdd37df03a4f993fee05c860f41cf61b

Changes applied:
  - Inference in "Why this is deferred": replaced `ImplementationContext … is {request, subject}` with `{request, subject, baseline: OnceCell<Option<BaselineImplementation>>}` and stated explicitly that no required-property / goal-property field exists and providers still receive no consumer requirement.
  - Replaced bare `:line` citations for `enumerate_frontier`, `ImplementationContext`, `derive_child_requirements`, `satisfy_edge`, `BoundaryContract::encode`, `encode_property_identity`, `PlanStructuralCost::dominates`, `SelectedPortfolio::non_dominated`, and the `#[cfg(test)] mod tests` open with file-path / symbol anchors.
  - Appended Trigger check log **2026-08-10 — not fired** at base `c99ac549` (recheck on live main): T1 unmet under file-local `derive_child_requirements` recheck; T2 unmet (enforcers deferred; 21 prose-only enforcer hits); noted `baseline` on ImplementationContext without a property goal; noted body line numbers from `d7b8604d` superseded by symbol anchors.

Optional items skipped (with reason):
  - none remaining: optional baseline note and line-number supersession note were included in the 2026-08-10 log entry as permitted.

Residuals not applied (docs/crates/new tickets/authority):
  - none for this audit (Exact files listed only this ticket). Eventual trigger-fire product work (`crates/tiler-compiler` goal + excluding parameter) is intentionally deferred and not in-scope for wave B.

Verification:
  - files read:
    - tickets/admit-an-excluding-property-parameter-when-a-goal-directed-input-search-lands.md (full, pre- and post-edit)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/admit-an-excluding-property-parameter-when-a-goal-directed-input-search-lands/45645b4786f0_c99ac54950f2.md (full)
    - crates/tiler-compiler/src/frontier.rs (`ImplementationContext` fields; `enumerate_frontier` signature)
    - crates/tiler-compiler/src/boundary.rs (`derive_child_requirements` call sites file-local only)
    - tickets/implement-boundary-property-enforcers.md (`status: deferred`)
  - checks:
    - `derive_child_requirements` hits only in `boundary.rs` (declaration + tests)
    - enforcer prose hit count 21 under `crates/**/*.rs`
    - `implement-boundary-property-enforcers` status deferred
    - shasum -a 256 of ticket after edit → post-edit hash above

Recommended next ledger state:
  integrated
