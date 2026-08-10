Ticket: name-the-elementary-identity-rewrite-dimension
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/name-the-elementary-identity-rewrite-dimension/f808b7978bc6_c99ac54950f2.md
Pre-edit content hash (from ledger): f808b7978bc6ef827c64ab6d86fc81cb978da1550c2770eb55fd619bfabd3e7e
Post-edit content hash: 7d3ce06a44ee7bd7656ea8df9391538774235e81bc28f9a93a047d1a209329ce

Changes applied:
  - Outcome final paragraph: rephrased the live present-tense claim that `SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM` registers `a-reassociation-of-the-sum-and-not-a-free-implementation-choice` as historical finding evidence (closed-at-research-time wording).
  - Added **Correction — 2026-08-10.** noting `correct-the-online-single-pass-softmax-fold-legality-fact` is `status: done` and quoting the current registered Horner/distributivity/elementary-identity string from `crates/tiler-ir/src/semantic/softmax.rs`, with a reproduce command.

Optional items skipped (with reason):
  - Optional one-line ADR 0101 acceptance / numerical-semantics normative note: not load-bearing for status; Outcome already points at the research record which states adoption; not graph hygiene on this ticket.

Residuals not applied (docs/crates/new tickets/authority):
  - Research-record internal tension ("What this record does not establish" vs ADR 0101 adoption) is outside this ticket file (docs residual).
  - Stale line citations inside the research record for accuracy.rs / request.rs / softmax.rs (docs residual).
  - No new remainder tickets required; permission admission, runtime measure, and related follow-ons already filed.

Verification:
  - files read:
    - tickets/name-the-elementary-identity-rewrite-dimension.md (full)
    - audit report f808b7978bc6_c99ac54950f2.md (full)
    - crates/tiler-ir/src/semantic/softmax.rs (SOFTMAX_F32_FACT_ONLINE_SINGLE_PASS_FORM registration; confirmed current multi-line fact string)
  - checks:
    - `shasum -a 256 tickets/name-the-elementary-identity-rewrite-dimension.md` → 7d3ce06a44ee7bd7656ea8df9391538774235e81bc28f9a93a047d1a209329ce
    - current fact string matches audit Fact 9 anchor `not-a-reassociation-of-the-sum-but-a-horner-nesting-consuming-distributivity-`
    - metadata unchanged (status done; related/scopes untouched)

Recommended next ledger state:
  integrated
