Ticket: restate-the-tree-width-rule-outside-the-compiler-crate
Wave: B5
Report: /Users/tsanterre/workspace/github.com/moderately-ai/tiler/docs/research/documentation/ticket-audit-2026-08-10/reports/restate-the-tree-width-rule-outside-the-compiler-crate/39b48f065b40_c99ac54950f2.md
Pre-edit content hash (from ledger): 39b48f065b4026ef5714e43c8cf2da1a2ce2e40d821f39e2a1d75b0fe7d18bf4
Post-edit content hash: 084d326133ebbcef39f91eb5f4cc088e4222c6acbe996136b131a2485f76fb34

Changes applied:
  - related: added `correct-the-two-participant-residue-s-smallest-count` (optional symmetric graph) and `correct-the-diverge-from-twelve-upward-phrasing-in-tests-and-proof` (new remainder)
  - ## Fact audit — 2026-08-10 dated correction naming the two residual "diverge from twelve contributors upward" sites (conformance tests.rs portfolio comment; prototype proof.rs declared_partition doc) and the preferred "first diverge at" fix language
  - Filed remainder ticket `tickets/correct-the-diverge-from-twelve-upward-phrasing-in-tests-and-proof.md` (todo, p3; scopes implementation/conformance + implementation/runtime; related back to this parent; comment-only closes-when)

Optional items skipped (with reason):
  - none (optional related edge to residue ticket applied as cheap graph hygiene)

Residuals not applied (docs/crates/new tickets/authority):
  - Product comment edits remain on the new remainder: `crates/tiler-conformance/src/serial_sum/tests.rs` and `prototypes/serial-sum-run/src/proof.rs` (wave B does not edit crates/prototypes)
  - Compiler residue "The smallest is 1,042" in `physical.rs` already owned by open `correct-the-two-participant-residue-s-smallest-count` (not re-filed)

Verification:
  - files read:
    - audit report (full)
    - tickets/restate-the-tree-width-rule-outside-the-compiler-crate.md (full, pre/post)
    - tickets/correct-the-two-participant-residue-s-smallest-count.md (frontmatter + body)
    - crates/tiler-conformance/src/serial_sum.rs declared_partition repaired "first diverge at" language
    - crates/tiler-conformance/src/serial_sum/tests.rs four-contributor portfolio comment (live "diverge from twelve")
    - prototypes/serial-sum-run/src/proof.rs declared_partition doc (live "diverge from twelve")
    - ticketsplease.toml scope rows for implementation/conformance and implementation/runtime
  - checks:
    - `rg 'diverge from twelve'` hits only the two residual product sites plus historical ticket prose
    - `rg 'first diverge at' crates/tiler-conformance/src/serial_sum.rs` confirms preferred fix language already landed on the parent Outcome site
    - shasum -a 256 on post-edit parent ticket
    - status left done; historical Fact table / What landed Outcome not rewritten

Recommended next ledger state:
  integrated
