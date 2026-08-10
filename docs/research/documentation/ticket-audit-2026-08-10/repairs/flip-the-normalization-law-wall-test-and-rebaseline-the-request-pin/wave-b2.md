Ticket: flip-the-normalization-law-wall-test-and-rebaseline-the-request-pin
Wave: B2
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/flip-the-normalization-law-wall-test-and-rebaseline-the-request-pin/e6100823cb8d_c99ac54950f2.md
Pre-edit content hash (from ledger): e6100823cb8d553fdf40443f47bcd30d872ec0301c64dc35e6b829924a294192
Post-edit content hash: 5b3d3a43ec2bce1d9ac4fc34f9e3e55b4c31d4fba73d1f1c9f90797adea96ecd

Changes applied:
  - Outcome delivery attribution: replaced `at the merge of d88ebdb8` with delivery commit `f33fa86eee6c2b77f97076175b88557557b1fb70` (parent merge `bdb3ae40`); d88ebdb8 is the unrelated scalar.rs link-definition commit, not the wall-test/pin repair.
  - Body site-2 pin direction: inverted live sentence from `b88654bff9b673c1 becomes ce6f9106c1c5933b` to `ce6f9106c1c5933b becomes b88654bff9b673c1` so it matches Outcome and git history (ce6f base → b886 observed at f33fa86e).

Optional items skipped (with reason):
  - none (report optional path was leave body historical + dated correction; preferred in-place invert was applied instead)

Residuals not applied (docs/crates/new tickets/authority):
  - Historical nextest population counts (2844 / 2847) not re-measured
  - Process narrative about empty-diff collision check against cover-atoms not re-run
  - Outcome still narrates the original transcription transposition in the request-pin paragraph (historical close-time note; body now matches the correct direction)

Verification:
  - files read:
      - audit report e6100823cb8d_c99ac54950f2.md
      - tickets/flip-the-normalization-law-wall-test-and-rebaseline-the-request-pin.md
      - git show f33fa86e / d88ebdb8 --stat; f33fa86e parents
  - checks:
      - f33fa86e edits explain.rs, two_region_occurrence_lowering.rs, scalar.rs, this ticket Outcome
      - d88ebdb8 touches only scalar.rs link placement (4 lines)
      - post-edit sha256 of ticket file

Recommended next ledger state:
  integrated
