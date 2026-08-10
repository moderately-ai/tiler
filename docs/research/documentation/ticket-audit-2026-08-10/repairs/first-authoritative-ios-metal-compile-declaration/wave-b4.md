Ticket: first-authoritative-ios-metal-compile-declaration
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/first-authoritative-ios-metal-compile-declaration/d07efc73dcc8_c99ac54950f2.md
Pre-edit content hash (from ledger): d07efc73dcc812a2392b477846a03883557f06adeeadbc29024c8aa920764598
Post-edit content hash: 226137ef2577bc1a46af7638ebf0e52939143967df0cda63c32dc3656f684888

Changes applied:
  - Replaced ledger bound source path with 2026-08-02 F32+BF16 unified MSL 4.0 record; re-anchored citation to searchable fragment `Both rows are transcribed from` (dropped stale `:51`).
  - Replaced inheritance “line 133” with searchable anchor `No iOS family, physical or simulated, gains a row from this one.`
  - Replaced `numerical-behaviour.md:219` with searchable anchor `the simulator result is admissible as a simulator measurement and not as an iOS-device one`.
  - Attributed `second_artifact_family_fixture` to `crates/tiler-build/src/metal_declaration.rs` (consumed by `metal_plan.rs` tests).
  - Spelled contract as `NumericalContract::FLUSH_SUBNORMALS_TO_ZERO_F32`; stopped claiming aot derives the one contract (regions state contracts; flushing contracts honour measured flush rows).
  - Added **Correction — 2026-08-10.** listing ledger path/line drift and fixture attribution fix.

Optional items skipped (with reason):
  - Optional dependency edge on `measure-apple-numerics-on-physical-ios-device` not added; report marks it optional graph hygiene only, not required for deferred posture correctness (related list already names it).

Residuals not applied (docs/crates/new tickets/authority):
  - none; Exact files expected only this ticket; no crate/docs product work.

Verification:
  - files read:
    - tickets/first-authoritative-ios-metal-compile-declaration.md (full, pre/post)
    - docs/research/documentation/ticket-audit-2026-08-10/reports/first-authoritative-ios-metal-compile-declaration/d07efc73dcc8_c99ac54950f2.md (full)
    - ledger source sentence at `Both rows are transcribed from` / inheritance at ledger F32 dispatchability row (rg)
    - numerical-behaviour finding-13 inference sentence (rg)
    - second_artifact_family_fixture defined in metal_declaration.rs, consumed in metal_plan.rs (rg)
  - checks:
    - shasum -a 256 on ticket after edit
    - rg confirms retired false anchors absent as live Facts; repaired anchors and Correction present

Recommended next ledger state:
  integrated
