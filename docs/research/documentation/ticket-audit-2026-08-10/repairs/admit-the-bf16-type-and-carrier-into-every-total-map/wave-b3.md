Ticket: admit-the-bf16-type-and-carrier-into-every-total-map
Wave: B3
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-the-bf16-type-and-carrier-into-every-total-map/45f81c3c2175_c99ac54950f2.md
Pre-edit content hash (from ledger): 45f81c3c21756f2839e059b83d8c9c477337258b5906240bae4aa5a5ed2549dd
Post-edit content hash: f7f89d041b8e73ea7659174d3edff440b3a5b1c645363eb135cf65ca4f0cf7a6

Changes applied:
  - User-visible outcome: rephrased "Nothing produces either variant yet" as non-goal at close of this ticket; named successor owners for production
  - Implementation key `msl_type`: present-tense refusal obligation → historical landing + supersession by lower-bf16-to-metal (`Ok("bfloat")` live arm)
  - Required evidence measurement boundary: labeled historical at close; Correction — 2026-08-10 that F32-only ScalarProgram / unconstructible VerifiedKernel is superseded by admit-bf16-into-the-schedule-and-kernel-vocabulary
  - Closes when: "all eight sites" → every total-map site then known / final eleven-site set matching re-run table
  - Graph maintenance: noted frontend scope addition from re-run (scheduling metadata already true in body)
  - Added ## Outcome for landing `129d783b` (variants, tags 0x06/0x03, five encoders, eleven sites, no identity step, artifact-abi no edit, non-goal / successors)
  - Added ## Fact audit — 2026-08-10 striking retired present-tense sentences and recording StorageScalar doc residual
  - status left `done`; no dependency frontmatter change

Optional items skipped (with reason):
  - none required optional graph hygiene left unapplied (related already lists spike; carry/lower already named in Graph)

Residuals not applied (docs/crates/new tickets/authority):
  - crates/tiler-ir/src/program/model.rs StorageScalar::Bf16 doc may still claim carrier-only / nothing-produces (source-doc drift; KernelType side addressed by correct-the-stale-bf16-backend-refusal-claim-in-the-kernel-type-doc; parallel StorageScalar doc repair is product/crates debt, not reopen of type admission)
  - exact make full terminal log path / pinned-identity recompute tables not recovered into Outcome (absent from pre-repair body; commit message states no identity move)

Verification:
  - files read:
    - tickets/admit-the-bf16-type-and-carrier-into-every-total-map.md (full, pre and post)
    - audit report 45f81c3c2175_c99ac54950f2.md (full)
    - crates/tiler-metal/src/emit.rs (msl_type Bf16 => Ok("bfloat") at live anchor)
    - git log / show 129d783b landing commit message and stat
    - tickets/lower-bf16-to-metal.md Outcome (supersession of refusal)
  - checks:
    - shasum -a 256 tickets/admit-the-bf16-type-and-carrier-into-every-total-map.md → f7f89d041b8e73ea7659174d3edff440b3a5b1c645363eb135cf65ca4f0cf7a6
    - rg confirms ## Outcome, Fact audit — 2026-08-10, struck "Nothing produces" / msl_type must refuse, live Ok("bfloat") citation
    - no crates/ or other tickets edited

Recommended next ledger state:
  integrated
