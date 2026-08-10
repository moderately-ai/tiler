Ticket: accept-the-neutral-build-orchestration-boundary
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/accept-the-neutral-build-orchestration-boundary/555dd52d4c78_c99ac54950f2.md
Pre-edit content hash (from ledger): 555dd52d4c7837962bdcbdff059ca1a4f82477d37f6be4a2145654289c355572
Post-edit content hash: cbe350978a4a5102d35a68c96c7dfc7ef1297162db7b43ad18292180fc2ad2da

Changes applied:
  - Why inventory claim: present-tense "none exists" / twenty-two-node `^accept` census reframed as **Historical (pre-this-ticket)**; notes this file is the acceptance node
  - Surface inventory: retired `lib.rs:80-84` line band → greppable `pub use payload_cache::{...}` / `pub use plan_artifact::{...}` (not metal_cache band)
  - Surface items: retired `plan_artifact.rs:152` / `:59` / `:77` and payload_cache line pins → `pub fn` / `pub struct` / `pub enum` symbol anchors in module files
  - Packet bound: retired `lib.rs:43-47` → "What remains bounded rather than neutral" + `the cache seam admits one payload per delivery position, shared by every`
  - Packet BindingKind: retired `model.rs:492` → `pub enum BindingKind` (sole variant `Buffer`)
  - Dropped stale ADR `:19` and promote-ticket `:79` line pins on related Facts (symbol/phrase anchors only)
  - Added `## Fact audit — 2026-08-10` dated correction summarizing inventory and line-pin rot
  - Metadata unchanged (status done, deps, scopes, related sound per report)

Optional items skipped (with reason):
  - none required beyond the dated Fact audit (report allowed rewrite as historical without separate note; both applied for house-style readability)

Residuals not applied (docs/crates/new tickets/authority):
  - none; report required no docs/crates edits and no remainder tickets; singleton BindingKind and one-payload-per-position remain accepted reservations

Verification:
  - files read: full audit report; full ticket; crates/tiler-build/src/lib.rs (pub use bands + bounded paragraph); plan_artifact.rs symbols; payload_cache.rs symbols; crates/tiler-artifact/src/program/model.rs `pub enum BindingKind` (Buffer only)
  - checks: shasum -a 256 post-edit ticket; rg confirms no live present-tense "none exists" inventory, no stale line pins as citations, greppable symbol anchors present

Recommended next ledger state:
  integrated
