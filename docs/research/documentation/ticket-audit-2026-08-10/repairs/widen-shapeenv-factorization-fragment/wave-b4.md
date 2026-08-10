Ticket: widen-shapeenv-factorization-fragment
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/widen-shapeenv-factorization-fragment/37375bad01f1_c99ac54950f2.md
Pre-edit content hash (from ledger): 37375bad01f1cdeb0938a559db4ea3058fdbb01f12fdcc4e1766ca5dadb4a51c
Post-edit content hash: 7dd3969c4f640edd34394fab5843f7777b15e6f1ba18fb6d31352bbbb574e977

Changes applied:
  - Replaced stale line citation `crates/tiler-ir/src/shape/env.rs:1447` with file path plus searchable anchor `a_relation_outside_the_supported_fragment_is_refused_rather_than_ignored` (verified at current tree: fn at env.rs:1637; still asserts `UnderdeterminedFactorization { undetermined: 3 }`).
  - Replaced stale line range `crates/tiler-ir/src/shape/env.rs:259-287` with anchor `pub enum BindingSource` in `crates/tiler-ir/src/shape/env.rs` (verified at current tree: enum at env.rs:276).
  - Added Trigger check log entry 2026-08-10 **not fired** (optional hygiene while the ticket was open for citation repair; recheck grep for the named test).

Optional items skipped (with reason):
  - none (optional trigger-log stamp applied as cheap hygiene on this same edit).

Residuals not applied (docs/crates/new tickets/authority):
  - none for this wave. Future activation (not audit repair) still owns durable refuse/support recording in shape contract / docs/ir.md and any Support-it implementation work; deferred status and close condition correctly unchanged.

Verification:
  - files read: audit report; full ticket; grep under crates/tiler-ir/src/shape for BindingSource, the refusal test, UnderdeterminedFactorization
  - checks: `grep -n 'a_relation_outside_the_supported_fragment_is_refused_rather_than_ignored' crates/tiler-ir/src/shape/env.rs` → env.rs:1637; `grep -n 'pub enum BindingSource' crates/tiler-ir/src/shape/env.rs` → env.rs:276; ticket no longer contains `:1447` or `:259-287`; shasum -a 256 of ticket file after edit

Recommended next ledger state:
  integrated
