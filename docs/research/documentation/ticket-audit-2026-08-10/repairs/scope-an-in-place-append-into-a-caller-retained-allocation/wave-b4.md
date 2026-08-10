Ticket: scope-an-in-place-append-into-a-caller-retained-allocation
Wave: B4
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/scope-an-in-place-append-into-a-caller-retained-allocation/b3e1833bc99f_c99ac54950f2.md
Pre-edit content hash (from ledger): b3e1833bc99f811447cfb011f37a963f1474c041999e19fd0c7e555a53fda8d0
Post-edit content hash: c19bdcd4c5b5a5519bab05db2ac26ff75b8c3bd302f6212d15d80f1f3f23b6e3

Changes applied:
  - related: added `admit-a-partitioned-write-ownership-contract` and `accept-the-partitioned-write-ownership-proof-boundary` (landed prerequisite graph hygiene)
  - "What it owes" item 3: replaced false claim that only `CoordinatePermutation`/`Exhaustive` exist and cannot express partition totality/disjointness; states `PartitionMember` proves joint partitioned **output** ownership; remaining gap is preservation of prior content in unwritten regions of a **caller-bound input** (plus still-standing ExternalValueWritten/ForbiddenAlias/recovery)
  - Intro framing: "Four implemented refusals" / proof-kind-absent wording adjusted so item 3 is not written as if the third proof kind is absent
  - **Correction — 2026-08-10.** after "What it owes" noting PartitionMember landed, remaining gap, conditions 2–3 unmet, trigger still not fired
  - Trigger check log 2026-08-10 — **not fired** with same PartitionMember / conditions 2–3 evidence
  - Status left `deferred` (trigger still not satisfied)

Optional items skipped (with reason):
  - none (optional related-edge add applied as cheap same-ticket graph hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - none required; report listed no remainder ticket and no docs/crates product edits for this repair
  - product work (measurement, recovery contract, verifier/alias relaxation if ever activated) remains deferred under the existing trigger — not wave B

Verification:
  - files read:
    - docs/research/documentation/ticket-audit-2026-08-10/reports/scope-an-in-place-append-into-a-caller-retained-allocation/b3e1833bc99f_c99ac54950f2.md
    - tickets/scope-an-in-place-append-into-a-caller-retained-allocation.md
    - crates/tiler-ir/src/index/model.rs (WriteOwnershipProof / WriteOwnershipProofView::PartitionMember)
    - crates/tiler-ir/src/program/verify.rs (ExternalValueWritten, MultipleWriters, ForbiddenAlias)
    - tickets/admit-a-partitioned-write-ownership-contract.md (status: done)
    - tickets/accept-the-partitioned-write-ownership-proof-boundary.md (status: done)
  - checks:
    - `rg PartitionMember crates/tiler-ir/src/index/model.rs` → enum + view present
    - `rg 'ExternalValueWritten|MultipleWriters|ForbiddenAlias' crates/tiler-ir/src/program/verify.rs` → three diagnostics still present
    - both partitioned-write tickets `status: done`
    - post-edit sha256 of ticket file

Recommended next ledger state:
  integrated
