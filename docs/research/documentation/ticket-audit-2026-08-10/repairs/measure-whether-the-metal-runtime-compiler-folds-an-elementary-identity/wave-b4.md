Ticket: measure-whether-the-metal-runtime-compiler-folds-an-elementary-identity
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/measure-whether-the-metal-runtime-compiler-folds-an-elementary-identity/9b57048cdbe9_c99ac54950f2.md
Pre-edit content hash (from ledger): 9b57048cdbe98df838e8b9c50e624f84bb12d69de5500b040749eb082fe671e9
Post-edit content hash: 32ca34a7893bef42371ebe744c8c86a2a78b9f89c3f554d303846689e0e8898b

Changes applied:
  - "Why this is deferred rather than todo": replaced "because the dimension is not admitted" with permission-unadmitted / no target-profile honourability declaration owed; noted dimension definition already under ADR 0101 / numerical-semantics (named-and-unpermissioned, outside `CANONICAL_DIMENSIONS`).
  - "Why this exists" finding-30 paraphrase: completed mode qualifier so the profile-wrong claim is "under `relaxed` or `fast`" (and the measured contraction is under `relaxed`/`fast`), matching the numerical-behaviour record inference.
  - Trigger: retuned gloss so declaration-owed means permission admission / addition to the declared honourability set profiles must answer, not "dimension admitted to the contract"; noted definition already in ADR 0101.
  - Metadata unchanged: status deferred, dependencies `[]`, related list, scopes as audited.

Optional items skipped (with reason):
  - 2026-08-10 trigger recheck log entry: optional; prior log rows remain correct and operational checks (CANONICAL_DIMENSIONS / permission ticket) still match the retuned trigger.

Residuals not applied (docs/crates/new tickets/authority):
  - none required by the report; no docs/crates edits; no remainder ticket; measurement remains deferred until a trigger fires.

Verification:
  - files read:
    - audit report `9b57048cdbe9_c99ac54950f2.md`
    - ticket (pre- and post-edit)
    - `crates/tiler-ir/src/numerics.rs` (`DIMENSION_COUNT` / `CANONICAL_DIMENSIONS`; no `ElementaryIdentity`)
    - finding-30 inference in `docs/research/apple-targets/numerical-behaviour.md` (mode-qualified clause present)
    - ADR 0101 decision 5 and numerical-semantics elementary-identity section (permission unadmitted; dimension defined)
  - checks:
    - `rg -c 'ElementaryIdentity' crates/tiler-ir/src/numerics.rs` → 0
    - ticket no longer contains "dimension is not admitted"
    - ticket contains `under \`relaxed\` or \`fast\`` and honourability-set / permission-admission trigger gloss
    - `shasum -a 256` of ticket after edit = post-edit hash above

Recommended next ledger state:
  integrated
