Ticket: correct-the-online-single-pass-softmax-fold-legality-fact
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/correct-the-online-single-pass-softmax-fold-legality-fact/1a49d72afda3_c99ac54950f2.md
Pre-edit content hash (from ledger): 1a49d72afda3dbca4545207d2289603094e0d65781edb1ec7191f613aba24293
Post-edit content hash: 4ccecdb5b6d7635df4ee65710c0111257d711e445dd9b894cda75d07323bc19b

Changes applied:
  - Why this exists: dated **Correction — 2026-08-10** frames the pre-`28fe26a8` Fact/Inference blocks as HISTORICAL (not live tree claims); added reproduce command for live refusal strings.
  - Outcome fact-grounds paragraph: dated **Correction — 2026-08-10** replaces live-false draft-ADR / empty-`docs/decisions/` / empty-grep claim with ADR 0101 accepted (2026-08-06) catalog-named unpermissioned status and accept-adr-0101 steward reading against typed `NumericalRealization` fields; fact string retained; empty-grep marked not current evidence.
  - Outcome pin enumeration: dated **Correction — 2026-08-10** marks `45467875b9574962` → `a95ad77532352d7f` as landing-time only; notes live pin `7ba3d77a66f04638` without rebaselining here.
  - Metadata (status/scopes/deps/related): unchanged as report required.

Optional items skipped (with reason):
  - none — recommended pin-pair landing-time note was cheap graph hygiene on this ticket and was applied.

Residuals not applied (docs/crates/new tickets/authority):
  - No code remainder filed: steward reading on accept-adr-0101 keeps fact string; catalog-vs-typed header absolute phrase (`no declared dimension names at all`) not reopened as identity work in wave B.
  - crates/tiler-ir/src/semantic/softmax.rs, softmax/tests.rs, crates/tiler-compiler/src/explain.rs: out of wave (ticket-prose only); only needed if typed-vs-catalog wording is reopened as a separate identity-domain ticket.
  - ADR 0101 open-questions prose still asserting the old reassociation fact: outside this ticket file; residual graph rot on that ADR, not repaired here.

Verification:
  - files read:
    - tickets/correct-the-online-single-pass-softmax-fold-legality-fact.md (full, pre- and post-edit)
    - audit report 1a49d72afda3_c99ac54950f2.md (full)
    - tickets/accept-adr-0101-elementary-identity-dimension.md (Decided steward sentence)
    - docs/decisions/0101-treat-elementary-function-identities-as-a-fourth-numerical-dimension.md (decision_status + Status accepted 2026-08-06)
    - crates/tiler-ir/src/semantic/softmax.rs (live fact registration + header refusal)
    - crates/tiler-compiler/src/explain.rs (live request pin 7ba3d77a66f04638)
  - checks:
    - `rg` live fact starts with not-a-reassociation-of-the-sum; old reassociation string absent from crates/softmax registration path
    - live explain pin `tiler-explain-v7 request=7ba3d77a66f04638`
    - shasum -a 256 post-edit ticket → 4ccecdb5b6d7635df4ee65710c0111257d711e445dd9b894cda75d07323bc19b

Recommended next ledger state:
  integrated
