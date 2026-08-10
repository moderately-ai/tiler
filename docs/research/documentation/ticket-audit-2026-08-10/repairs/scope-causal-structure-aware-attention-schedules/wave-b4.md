Ticket: scope-causal-structure-aware-attention-schedules
Wave: B4
Report: docs/research/documentation/ticket-audit-2026-08-10/reports/scope-causal-structure-aware-attention-schedules/f8d1092a9ac9_c99ac54950f2.md
Pre-edit content hash (from ledger): f8d1092a9ac91ad9b42d6a9826c9037070d8c39e84fdc338a159cf6b9214ccae
Post-edit content hash: 8dbb7b81bf43dde4acdf6dd714afac43066bce16f80ca7ca8913fde7a6b8fe9c

Changes applied:
  - Replaced false "no permission in the numerical contract covers a signed-zero rewrite" with: contract declares existing `signed_zero` dimension with identity; under `signed_zero: Forbidden` (workload strict F32) skipping is forbidden; open half is whether `Permitted` licenses contributor omission vs ±0 result equivalence only.
  - Replaced "the three numerical dimensions" with explicit inventory: eleven governed dimensions including reassociation, permutation, and existing `signed_zero`; distributivity and elementary-function identity named with no permission field; each route must name field consumed, invented non-field, or Softmax meaning change.
  - Reframed candidate route 1 from "a declared signed-zero relaxation with its own identity" to granting already-declared `signed_zero` (`Permitted`) under ADR 0011 and stating whether that alone authorizes contributor-sequence omission (identity/explain included); route 1 must not invent a new dimension.
  - Status/deps/scopes/related left unchanged (report: graph sound, none required).

Optional items skipped (with reason):
  - Dated correction block: skipped; body rewritten cleanly without preserving the false universal claim (report marked optional when clean rewrite).

Residuals not applied (docs/crates/new tickets/authority):
  - Later product execution of this research ticket may still touch `docs/numerical-semantics.md`, an ADR under `docs/decisions/`, L4 D-9 closure in `docs/research/program-planning/first-attention-program-vertical.md`, and/or Softmax definition docs — outcome-dependent; not wave-B prose repair.
  - Residual uncertainty from report remains the ticket's open research: whether any existing legality/explain path treats `signed_zero: Permitted` as authorizing contributor omission (no attention skip rule exists today).

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/scope-causal-structure-aware-attention-schedules/f8d1092a9ac9_c99ac54950f2.md (full)
    - tickets/scope-causal-structure-aware-attention-schedules.md (full, pre- and post-edit)
    - crates/tiler-ir/src/numerics.rs (`NumericalDimension::SignedZero` in CANONICAL_DIMENSIONS)
    - crates/tiler-ir/src/schedule/numerics.rs (`signed_zero`, `permits_signed_zero_elimination`)
    - docs/numerical-semantics.md (eleven governed dimensions; `signed_zero: NumericalPermission`)
  - checks:
    - `rg 'no permission in the numerical contract covers|three numerical dimensions|declared signed-zero relaxation with its own identity' tickets/scope-causal-structure-aware-attention-schedules.md` → empty
    - `rg 'already-declared .signed_zero.|eleven.*governed dimensions|contributor-sequence omission|signed_zero: Forbidden' tickets/scope-causal-structure-aware-attention-schedules.md` → hits on repaired prose
    - `shasum -a 256 tickets/scope-causal-structure-aware-attention-schedules.md` → 8dbb7b81bf43dde4acdf6dd714afac43066bce16f80ca7ca8913fde7a6b8fe9c

Recommended next ledger state:
  integrated
