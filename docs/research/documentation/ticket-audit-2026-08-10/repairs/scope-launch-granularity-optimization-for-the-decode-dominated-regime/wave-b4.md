Ticket: scope-launch-granularity-optimization-for-the-decode-dominated-regime
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/scope-launch-granularity-optimization-for-the-decode-dominated-regime/44c373eba4af_c99ac54950f2.md
Pre-edit content hash (from ledger): 44c373eba4af6cc987151551f6c0e7f81f82892581c1df58b5c6538ec09581b6
Post-edit content hash: 79d103eb3c537ab7cf592ef623f5fe10dbe06941c9424d8b5de398c278b095aa

Changes applied:
  - Rewrote compound Fact: 62 is one P2 layer graph at C1 decode T=1 (58 prefill); three programs with 30 executions/pass (P2×28, P1/P3×1); occurrences are not launches; stage-DAG Fact split out.
  - Added dated Correction — 2026-08-10 recording the conflation of occurrence count, program partition, and layer-loop multiplicity.
  - Replaced "Deferred behind the multi-layer execution work" with language matching Trigger disjuncts (multi-stage e2e or multi-layer model run).
  - Outcome: "persistent/mega-kernel forms" → "fused / multi-stage program forms" with glossary caveat on mega-kernel.
  - Added ## Required analysis, ## Non-goals, and ## Closes when (measurement plan + named surface + remainders; four research exits).
  - Left status todo, dependency prototype-metal-runtime-proof, related decide-ticket, scopes, tags (including trigger-fired) unchanged.

Optional items skipped (with reason):
  - related assemble-the-decoder-layer-program or L6: optional; repaired Fact cites decoder_layer.rs pins and L6 numbers without hard graph edges; audit said not mandatory and warned against re-parking behind unfinished model execution.

Residuals not applied (docs/crates/new tickets/authority):
  - none required for this wave (Exact files were ticket-only; no crate/ADR edits; no new remainder tickets until scoping runs).

Verification:
  - files read: audit report; full ticket; decoder_layer.rs assert pins 58/62; L6 Executions per forward pass table (P2=28); glossary mega-kernel; peer scope-precision (thin form) and scope-causal (Closes when form)
  - checks: shasum -a 256 of ticket after edit

Recommended next ledger state:
  integrated
