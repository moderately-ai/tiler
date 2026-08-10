Ticket: scope-the-remaining-elementwise-float-algebra-families
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-the-remaining-elementwise-float-algebra-families/61eacd48f2cb_c99ac54950f2.md
Pre-edit content hash (from ledger): 61eacd48f2cb9510b26905a497dd123f7cceaf3bdc9b3cc054ce34d50f408436
Post-edit content hash: 58a48d02fe5c8e46f2695aba90a25c09d128d21ecbb754a4ab87a325aadd2029

Changes applied:
  - Fact paragraph: matrix now stated as remaining-algebra R2 row (`Subtract`/`Divide`/negation) plus separate Fma R2 row (2026-08-05 split); dropped "one row … or Fma" evidence wording.
  - Replaced every Q-SEM-001 / "Q-SEM-004's sibling" reciprocal authority cite with numerical-semantics `reciprocal_transform` and explicit non-inheritance from silu/rms_norm/softmax formula pins (Activation trigger, Inference, What the work would be, Closes when).
  - Aligned unary scope to O-17 = full F-05 (`abs`, `negate`, `sign`) + F-06 subtract/divide; user-visible outcome, Inference, trigger, Closes when, and 2026-08-10 trigger-log entry cover abs/sign; noted matrix row title names only negation among unaries.
  - Softened "four accepted ADRs" to named ADR 0024 rounding authority (partial implementation; special values still owed).
  - Optional precision applied: "no **semantic** standard-registry key"; noted scalar `tiler.scalar::divide-f32@1` exists for composites.
  - Added **Correction — 2026-08-10.** recording matrix Fma split, Q-SEM-001 close, ADR count drop, and semantic-vs-scalar key precision.
  - Trigger check log: marked 2026-08-05 census integers historical; added 2026-08-10 not-fired recheck covering full F-05 and semantic-key absence.
  - Metadata unchanged (status deferred, deps, related, scopes, tags).

Optional items skipped (with reason):
  - none (optional semantic-vs-scalar "no key" precision was cheap and applied).

Residuals not applied (docs/crates/new tickets/authority):
  - docs/roadmap.md remaining-algebra matrix **trigger** cell still says reciprocal permission "resolved under Q-SEM-001" (ticket quotes that staleness; wave B forbids docs/ edits — residual product debt called out in Activation trigger Note).
  - No new remainder tickets; no crates/ or ADR work.

Verification:
  - files read:
    - tickets/scope-the-remaining-elementwise-float-algebra-families.md (full, pre/post)
    - audit report 61eacd48f2cb_c99ac54950f2.md (full)
    - docs/roadmap.md remaining-algebra + Fma matrix rows (full line 483 trigger column)
    - docs/open-questions.md Q-SEM-001 closed entry
    - docs/numerical-semantics.md reciprocal_transform / composite formula pins
    - docs/research/semantic-graph/operation-family-delivery-graph.md O-17 / F-05–F-06 mapping
    - crates/tiler-ir/src/index/scalar.rs divide_f32_scalar_op (grep)
  - checks:
    - Q-SEM-001 closed as presets supersession (open-questions.md)
    - matrix: separate remaining-algebra and Fma rows; Fma split fact 2026-08-05
    - O-17 covers F-05 whole + F-06 subtract/divide
    - scalar divide-f32 exists; no product implementation performed

Recommended next ledger state:
  integrated
