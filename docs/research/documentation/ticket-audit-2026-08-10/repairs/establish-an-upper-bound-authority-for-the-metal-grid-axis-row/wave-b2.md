Ticket: establish-an-upper-bound-authority-for-the-metal-grid-axis-row
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/establish-an-upper-bound-authority-for-the-metal-grid-axis-row/5c81d380d98f_c99ac54950f2.md
Pre-edit content hash (from ledger): 5c81d380d98f01e3d31697e39963acb12e719adc81e3272ff4e694216bf47c19
Post-edit content hash: 8f83caea1035864b2edb8d73e52414ba0f1197aaf485762eb13987960ab144e9

Changes applied:
  - Why: dated **Correction — 2026-08-10** banner marking Measurement/Fact/Inference as pre-landing problem statement; points at live `grid_axis_threads: 268_435_456` and `the_measured_grid_axis_admits_more_than_one_three_strategy_shape`; drops rotted `metal_declaration.rs:185-188`.
  - Why Measurement 2026-08-02 past-tensed (was floor / collapsed / retained / was refused).
  - Why Fact struck as live claim (`~~Fact~~`); past-tensed pre-landing four-thread declaration; notes live Metal measured bound and unmoved prototype baseline.
  - Why Inference past-tensed (was / then-current).
  - What blocks: dated correction that calibrate is done and the signal test was renamed/relocated; bullets past-tensed as pre-landing graph.
  - Outcome: clarified candidate-shape list as membership probe, not pinned complete domain equality.
  - related: added `raise-the-metal-grid-axis-row-to-reach-the-l3-contraction-cells` (optional graph hygiene; child depends on this parent).

Optional items skipped (with reason):
  - none remaining (optional candidate-shapes clarification and related-list hygiene both applied).

Residuals not applied (docs/crates/new tickets/authority):
  - none for this ticket's close condition. Report explicitly does not re-open for later identity pin drift or cost-row descriptor growth (owned outside).
  - SDK header hashes / Xcode 27A5228h remain UNVERIFIABLE on this host (no product edit required).
  - Live metal_plan pins vs authority-ledger "What those pins are today" drift is outside this ticket.

Verification:
  - files read:
    - tickets/establish-an-upper-bound-authority-for-the-metal-grid-axis-row.md
    - docs/research/documentation/ticket-audit-2026-08-10/reports/establish-an-upper-bound-authority-for-the-metal-grid-axis-row/5c81d380d98f_c99ac54950f2.md
    - crates/tiler-build/src/metal_declaration.rs (`grid_axis_threads: 268_435_456`)
    - crates/tiler-build/src/metal_plan.rs (`the_measured_grid_axis_admits_more_than_one_three_strategy_shape`, candidates array)
    - crates/tiler-compiler/src/target.rs (`declare_max_threads_per_grid_axis(4, …)` on governed; `the_prototype_baseline_admits_one_three_strategy_shape`)
    - tickets/calibrate-and-activate-parallel-reduction-selection.md (status done; dependency edge)
  - checks:
    - `rg grid_axis_threads crates/tiler-build/src/metal_declaration.rs` → `268_435_456` on FIRST_MACOS_APPLE9
    - `rg 'let candidates' crates/tiler-build/src/metal_plan.rs` → membership list not domain equality
    - ticket no longer present-tenses calibrate as blocked or cites `metal_declaration.rs:185-188` as live
    - `shasum -a 256` on ticket after edit

Recommended next ledger state:
  integrated
