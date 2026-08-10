Ticket: decide-the-unnameable-gpu-enumerator-channel
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/decide-the-unnameable-gpu-enumerator-channel/d65c693f934f_c99ac54950f2.md
Pre-edit content hash (from ledger): d65c693f934ff02e33987caf79abdfd0533c34a8063db3a3c8fa8c7b4b99b21b
Post-edit content hash: 5903a12076a3f47ca3b077f8cecfbd6f7041fe4a439100070dc2a6e32bd3341f

Changes applied:
  - status: deferred → todo; tags: drop deferred, add trigger-fired
  - Fact "exactly one of the two bindings": rewritten as three members / two metal-enum consumers (serial-sum-run + tiler-conformance) + one objc2-metal consumer, with dated Correction
  - Local-refusal Fact: cites both serial-sum-run and tiler-conformance probe sites
  - Recommendation / Inference: "exactly one instance" dated; second-instance rediscovery language updated; 2026-08-10 note that trigger A fired and board is todo
  - Closes when: consumer list includes tiler-conformance; claim-time scopes note for runtime/conformance/candle; "record on widen and this closes" struck with dated supersession by 2026-08-09 carrier
  - Deferral: dated correction that trigger A fired; exact signature still Tom's
  - Trigger check log: 2026-08-04 entry annotated recheck blind to crates/; 2026-08-10 **fired** (A) / B not fired with covering recheck commands

Optional items skipped (with reason):
  - related frontmatter edge for tiler-conformance: no ticket id owns "conformance is second metal consumer"; named in body Facts instead (report allowed related or body)

Residuals not applied (docs/crates/new tickets/authority):
  - implementation of fallible observe channel in tiler-metal + consumer migrations (serial-sum-run, tiler-conformance, candle adapter) — product work, not wave B
  - exact Result vs Option<bool> / third-outcome public signature (Tom under ADR 0074 §7) — authority residual; ticket now todo as carrier, not re-opening the binary deferral question
  - scopes not pre-expanded on board (report: add at claim time)

Verification:
  - files read: audit report d65c693f934f_c99ac54950f2.md; full ticket pre-edit; rg over tiler-conformance and serial-sum-run for ProbedGpuFamily/binding_apple_enumerator/COUNT; Cargo.toml metal/objc2-metal edges; widen status: deferred
  - checks: both metal consumers present; candle still objc2-metal; widen deferred (B not fired)

Recommended next ledger state:
  integrated
