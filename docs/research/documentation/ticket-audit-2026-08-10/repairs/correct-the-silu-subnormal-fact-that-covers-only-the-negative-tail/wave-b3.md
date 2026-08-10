Ticket: correct-the-silu-subnormal-fact-that-covers-only-the-negative-tail
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/correct-the-silu-subnormal-fact-that-covers-only-the-negative-tail/95ca1d2f981d_c99ac54950f2.md
Pre-edit content hash (from ledger): 95ca1d2f981d709fbcd6712882697dd26e89734984be3f7aaf0ac3ca7c5cab37
Post-edit content hash: a945a0c3ca0ce4ae2b3b3e3472f1fd05901b0a9556966bca0980eb9f43797be9

Changes applied:
  - Why this exists: dated **Correction — 2026-08-10** striking the false present-tense Fact that `SILU_F32_FACT_SUBNORMALS` still resolves to the unreachable spelling; points at live domain-true registration and notes stale `silu.rs:110` citation.
  - Outcome: dated **Correction — 2026-08-10** marking Outcome line citations and the intermediate explain pin transition `f3244b2242ebcb5c` → `6dd42be71c6745fe` (plus red-gate counts / present-tense red-gate sentence) as branch-landing history; live pin is `request=7ba3d77a66f04638`; recompute ticket `done`.
  - Outcome end: dated residual note for unsplit `silu_f32_under` rustdoc rot in `crates/tiler-reference/src/silu.rs` (evaluation still correct; docs inverted vs corrected fact); no related link yet pending remainder ticket id.

Optional items skipped (with reason):
  - none of the report's optional bullets were skipped; the optional dated correction on line citations / intermediate pin was applied as cheap same-ticket prose hygiene. RmsNorm sibling wording imprecision (value string lacks `reached` token) was audit Note only, not a Repair required bullet.

Residuals not applied (docs/crates/new tickets/authority):
  - NEW remainder ticket (narrow): rewrite `silu_f32_under` rustdoc in `crates/tiler-reference/src/silu.rs` so it no longer claims the declared fact "does not cover" reachability or that the fact only records the large-negative tail; align with current `SILU_F32_FACT_SUBNORMALS`; preserve still-true claim that the evaluator applies both subnormal dimensions. Scope maps to `crates/tiler-reference/**`, not `implementation/ir`. Report requires filing + related edge; wave B forbids inventing a ticket id and forbids crates/docs product edits — blocked residual until a concrete id is chosen and product work is dispatched.
  - `related` link from this ticket to that remainder — deferred until remainder is filed.
  - Exact files for product remainder: `crates/tiler-reference/src/silu.rs` (stale docs only; identity does not fold rustdoc).

Verification:
  - files read:
    - tickets/correct-the-silu-subnormal-fact-that-covers-only-the-negative-tail.md (full, pre- and post-edit)
    - audit report 95ca1d2f981d_c99ac54950f2.md (full)
    - crates/tiler-reference/src/silu.rs (`silu_f32_under` rustdoc slice; anchor `fact does not cover the case that reaches them`)
    - crates/tiler-ir/src/semantic/silu.rs (live fact registration via grep for `preserved-by-this-contract-and-reached-as-a-result-near-zero`)
    - crates/tiler-compiler/src/explain.rs (sole live pin `request=7ba3d77a66f04638`)
  - checks:
    - `rg` live SiLU fact spelling present in silu.rs + tests; unreachable not a live registration
    - `rg` reference rustdoc still has pre-repair "does not cover" claim
    - sole explain request pin is `7ba3d77a66f04638`
    - post-edit `shasum -a 256` of the ticket file

Recommended next ledger state:
  integrated
