Ticket: admit-a-multi-region-index-realization-law
Wave: B1
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/admit-a-multi-region-index-realization-law/4bb03e9368e0_c99ac54950f2.md
Pre-edit content hash (from ledger): 4bb03e9368e0377fe8ce9fb5e349ac75d55a947d6ddbc700c119f409c83d87c5
Post-edit content hash: abd9085a561de882118f148e54926d759912a1180573d74e8f73f6f5fad833fb

Changes applied:
  - Why this is filed: dated the wall binary path as historical (`two_region_occurrence_lowering_wall.rs` → live `two_region_occurrence_lowering.rs`) with Correction (2026-08-10).
  - Why this is filed: labelled the nine-operation / absent-normalization-and-softmax Fact as discovery-time and corrected the live census to fifteen rows including RMS and softmax (Correction 2026-08-10).
  - Why this is filed: labelled single-region realize/verify Fact as discovery-time and corrected staged refuse + realize_sequence/verify_sequence path (Correction 2026-08-10).
  - Outcome: marked wall-test MissingRealizationLaw assertion as discovery-time / historical path; confirmed tag-9 still unregistered on standard while tags 10–11 landed later (Correction 2026-08-10).
  - Outcome: rewrote the ten-scalar / no-rsqrt Inference as at-close history; corrected live rsqrt/maximum admission and RMS law registration (Correction 2026-08-10).
  - Correctness: rewrote present-tense retention prose to multi-reader / any-earlier-producer + retained_through; dated the immediate-only / one-reader contract as close-time, superseded 2026-08-06 multi-reader acceptance (Correction 2026-08-10).
  - Metadata left unchanged (status/deps/scopes already correct per report).

Optional items skipped (with reason):
  - none (report listed no optional prose/graph items; related list left as-is).

Residuals not applied (docs/crates/new tickets/authority):
  - none required by report; Exact files was ticket-only. No docs/crates edits; no new remainder tickets (report: none).

Verification:
  - files read:
    - tickets/admit-a-multi-region-index-realization-law.md (full, pre and post)
    - audit report 4bb03e9368e0_c99ac54950f2.md (full)
    - crates/tiler-ir/src/semantic/registry.rs (standard law loop; fifteen rows incl. RMS/softmax)
    - crates/tiler-ir/src/index/scalar.rs (public *_scalar_op census incl. rsqrt/maximum)
    - crates/tiler-ir/src/index/sequence.rs (header retention contract; any-earlier / retained_through)
    - crates/tiler-ir/src/index/law.rs (grep: No standard operation carries this row; staged constructors)
    - crates/tiler-compiler/tests/two_region_occurrence_lowering.rs (rename + history header)
  - checks:
    - shasum -a 256 tickets/admit-a-multi-region-index-realization-law.md → abd9085a561de882118f148e54926d759912a1180573d74e8f73f6f5fad833fb
    - rg Correction \(2026-08-10\) on ticket: five dated blocks
    - no metadata change; status remains done

Recommended next ledger state:
  integrated
